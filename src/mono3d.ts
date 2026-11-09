/**
 * AIDO-MONO 3D Character Viewer (WebGL 2.0 / 1.0)
 *
 * Zero-dependency, offline-first WebGL renderer for /assets/AIDO-MONO.glb
 * Parses binary glTF 2.0 (magic 0x46546C67), extracts the 3 articulated nodes:
 *   - Node 0: "MONO · O head" (1.832 Vertices, 9.348 Indices)
 *   - Node 1: "MONO · D chassis and A legs" (1.372 Vertices, 6.912 Indices)
 *   - Node 2: "MONO · I core" (68 Vertices, 276 Indices)
 * Total: 3.272 Vertices / 16.536 Indices
 *
 * Implements independent 3-node articulation and Masterplan Chapter 10 state animations:
 *   - 'sleep': lowered O-head, deep slow sleep breathing, dimmed core
 *   - 'wake': smooth upward head arc & welcoming nod, bright core flare
 *   - 'idle': gentle vertical hover & crisp breathing pulse
 *   - 'listen' / 'listening': attentive head tilt & responsive white/silver core pulse
 *   - 'think' / 'thinking': thoughtful scanning yaw orbit & rhythmic core oscillation
 *   - 'work': focused double-time articulation & precision core pulse
 *   - 'speak' / 'speaking': natural speech cadence nod
 *   - 'success': affirmative double-nod & upward lift
 *   - 'error': clear horizontal head-shake & high-contrast alert pulse
 *
 * Pure Monochrome Studio Lighting (Light Mode #FFFFFF & Dark Mode #000000).
 * Direct Character Mouse Gestures: Short Click, Hold (Push-to-Talk), Drag 360°, Middle-Click, Right-Click.
 */

export type MonoState =
  | "sleep"
  | "wake"
  | "idle"
  | "listen"
  | "listening"
  | "think"
  | "thinking"
  | "work"
  | "speak"
  | "speaking"
  | "success"
  | "error";

export interface MonoStateMeta {
  id: MonoState;
  label: string;
  description: string;
}

export const MONO_STATES_META: MonoStateMeta[] = [
  {
    id: "sleep",
    label: "Schlafen",
    description: "Ruhezustand · Kopf gesenkt, ruhige Atmung",
  },
  {
    id: "wake",
    label: "Aufwachen",
    description: "Aktivierung · Kopf hebt sich mit kurzem Nicken",
  },
  {
    id: "idle",
    label: "Bereit",
    description: "Aufmerksam · Sanftes Schweben, bereit für Eingaben",
  },
  {
    id: "listen",
    label: "Zuhören",
    description: "Sprachaufnahme · Kopf leicht geneigt, Kern pulsiert",
  },
  {
    id: "think",
    label: "Nachdenken",
    description: "Analyse · Nachdenkliche Kopfbewegung, Kern oszilliert",
  },
  {
    id: "work",
    label: "Arbeiten",
    description: "Ausführung · Fokussierte Artikulation bei aktiver Aufgabe",
  },
  {
    id: "success",
    label: "Erfolg",
    description: "Bestätigung · Zufriedenes Nicken nach Abschluss",
  },
  {
    id: "error",
    label: "Fehlermeldung",
    description: "Hinweis · Klares Kopfschütteln bei Rückfrage oder Fehler",
  },
];

export function getMonoStateMeta(state: MonoState): MonoStateMeta {
  const normalized: MonoState =
    state === "listening"
      ? "listen"
      : state === "thinking"
        ? "think"
        : state === "speaking"
          ? "idle"
          : state;
  return (
    MONO_STATES_META.find((m) => m.id === normalized) || MONO_STATES_META[2]
  );
}

export interface MonoGestureCallbacks {
  onShortClick?: (event: MouseEvent | PointerEvent) => void;
  onHoldStart?: (event: PointerEvent) => void;
  onHoldEnd?: (event: PointerEvent, durationMs: number) => void;
  onMiddleClick?: (event: MouseEvent | PointerEvent) => void;
  onRightClick?: (event: MouseEvent) => void;
}

export interface NodeMeshStats {
  name: string;
  nodeIndex: number;
  vertices: number;
  indices: number;
}

interface ParsedNodeMesh {
  name: string;
  nodeIndex: number;
  positionBuffer: WebGLBuffer;
  normalBuffer: WebGLBuffer;
  indexBuffer: WebGLBuffer;
  indexCount: number;
  vertexCount: number;
  centroid: [number, number, number]; // post-rotation (Y-up)
}

// Matrix helper utilities (Column-major Float32Array)
function mat4Identity(): Float32Array {
  return new Float32Array([
    1, 0, 0, 0,
    0, 1, 0, 0,
    0, 0, 1, 0,
    0, 0, 0, 1,
  ]);
}

function mat4Multiply(out: Float32Array, a: Float32Array, b: Float32Array): Float32Array {
  const a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
  const a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
  const a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
  const a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];

  let b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3];
  out[0] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
  out[1] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
  out[2] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
  out[3] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

  b0 = b[4]; b1 = b[5]; b2 = b[6]; b3 = b[7];
  out[4] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
  out[5] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
  out[6] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
  out[7] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

  b0 = b[8]; b1 = b[9]; b2 = b[10]; b3 = b[11];
  out[8] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
  out[9] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
  out[10] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
  out[11] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

  b0 = b[12]; b1 = b[13]; b2 = b[14]; b3 = b[15];
  out[12] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
  out[13] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
  out[14] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
  out[15] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
  return out;
}

function mat4Translate(x: number, y: number, z: number): Float32Array {
  const m = mat4Identity();
  m[12] = x;
  m[13] = y;
  m[14] = z;
  return m;
}

function mat4Scale(x: number, y: number, z: number): Float32Array {
  const m = mat4Identity();
  m[0] = x;
  m[5] = y;
  m[10] = z;
  return m;
}

function mat4RotateX(rad: number): Float32Array {
  const m = mat4Identity();
  const c = Math.cos(rad);
  const s = Math.sin(rad);
  m[5] = c;
  m[6] = s;
  m[9] = -s;
  m[10] = c;
  return m;
}

function mat4RotateY(rad: number): Float32Array {
  const m = mat4Identity();
  const c = Math.cos(rad);
  const s = Math.sin(rad);
  m[0] = c;
  m[2] = -s;
  m[8] = s;
  m[10] = c;
  return m;
}

function mat4RotateZ(rad: number): Float32Array {
  const m = mat4Identity();
  const c = Math.cos(rad);
  const s = Math.sin(rad);
  m[0] = c;
  m[1] = s;
  m[4] = -s;
  m[5] = c;
  return m;
}

function mat4Perspective(fovyRad: number, aspect: number, near: number, far: number): Float32Array {
  const out = new Float32Array(16);
  const f = 1.0 / Math.tan(fovyRad / 2);
  const nf = 1 / (near - far);
  out[0] = f / aspect;
  out[5] = f;
  out[10] = (far + near) * nf;
  out[11] = -1;
  out[14] = 2 * far * near * nf;
  return out;
}

function mat4LookAt(eye: [number, number, number], center: [number, number, number], up: [number, number, number]): Float32Array {
  const [eyex, eyey, eyez] = eye;
  const [cx, cy, cz] = center;
  const [upx, upy, upz] = up;

  let z0 = eyex - cx;
  let z1 = eyey - cy;
  let z2 = eyez - cz;
  let len = 1 / Math.hypot(z0, z1, z2);
  z0 *= len; z1 *= len; z2 *= len;

  let x0 = upy * z2 - upz * z1;
  let x1 = upz * z0 - upx * z2;
  let x2 = upx * z1 - upy * z0;
  len = Math.hypot(x0, x1, x2);
  if (!len) {
    x0 = 1; x1 = 0; x2 = 0;
  } else {
    len = 1 / len;
    x0 *= len; x1 *= len; x2 *= len;
  }

  let y0 = z1 * x2 - z2 * x1;
  let y1 = z2 * x0 - z0 * x2;
  let y2 = z0 * x1 - z1 * x0;
  len = 1 / Math.hypot(y0, y1, y2);
  y0 *= len; y1 *= len; y2 *= len;

  const out = new Float32Array(16);
  out[0] = x0; out[1] = y0; out[2] = z0; out[3] = 0;
  out[4] = x1; out[5] = y1; out[6] = z1; out[7] = 0;
  out[8] = x2; out[9] = y2; out[10] = z2; out[11] = 0;
  out[12] = -(x0 * eyex + x1 * eyey + x2 * eyez);
  out[13] = -(y0 * eyex + y1 * eyey + y2 * eyez);
  out[14] = -(z0 * eyex + z1 * eyey + z2 * eyez);
  out[15] = 1;
  return out;
}

function mat3NormalFromMat4(out: Float32Array, a: Float32Array): Float32Array {
  const a00 = a[0], a01 = a[1], a02 = a[2];
  const a10 = a[4], a11 = a[5], a12 = a[6];
  const a20 = a[8], a21 = a[9], a22 = a[10];

  const b01 = a22 * a11 - a12 * a21;
  const b11 = -a22 * a10 + a12 * a20;
  const b21 = a21 * a10 - a11 * a20;

  let d = a00 * b01 + a01 * b11 + a02 * b21;
  if (!d) return out;
  d = 1.0 / d;

  out[0] = b01 * d;
  out[1] = (-a22 * a01 + a02 * a21) * d;
  out[2] = (a12 * a01 - a02 * a11) * d;
  out[3] = b11 * d;
  out[4] = (a22 * a00 - a02 * a20) * d;
  out[5] = (-a12 * a00 + a02 * a10) * d;
  out[6] = b21 * d;
  out[7] = (-a21 * a00 + a01 * a20) * d;
  out[8] = (a11 * a00 - a01 * a10) * d;
  return out;
}

// GLSL Shaders
const VS_SOURCE = `
attribute vec3 a_position;
attribute vec3 a_normal;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;
uniform mat3 u_normalMatrix;

varying vec3 v_normal;
varying vec3 v_worldPos;

void main() {
  vec4 worldPos = u_model * vec4(a_position, 1.0);
  v_worldPos = worldPos.xyz;
  v_normal = normalize(u_normalMatrix * a_normal);
  gl_Position = u_projection * u_view * worldPos;
}
`;

const FS_SOURCE = `
precision mediump float;

varying vec3 v_normal;
varying vec3 v_worldPos;

uniform vec3 u_baseColor;
uniform vec3 u_emissiveColor;
uniform float u_metallic;
uniform float u_roughness;
uniform float u_rimIntensity;
uniform vec3 u_cameraPos;

void main() {
  vec3 N = normalize(v_normal);
  // Double-sided normal correction
  vec3 V = normalize(u_cameraPos - v_worldPos);
  if (dot(N, V) < 0.0) {
    N = -N;
  }

  // Pure Monochrome 3-Point Studio Lighting (Key, Fill, Rim)
  // Key Light: crisp upper-right-front
  vec3 L_key = normalize(vec3(1.4, 2.0, 2.2));
  vec3 C_key = vec3(1.0, 1.0, 1.0) * 1.15;

  // Fill Light: neutral monochrome lower-left-front
  vec3 L_fill = normalize(vec3(-1.8, -0.4, 1.2));
  vec3 C_fill = vec3(0.72, 0.72, 0.74) * 0.58;

  // Rim Light: high sharp architectural back-light
  vec3 L_rim = normalize(vec3(0.0, 2.2, -2.4));
  vec3 C_rim = vec3(1.0, 1.0, 1.0) * u_rimIntensity;

  // Ambient lighting for architectural obsidian / satin titanium tones
  vec3 ambient = vec3(0.24, 0.24, 0.25) * u_baseColor;

  // Diffuse components
  float diffKey = max(dot(N, L_key), 0.0);
  float diffFill = max(dot(N, L_fill), 0.0);
  vec3 diffuse = (diffKey * C_key + diffFill * C_fill) * u_baseColor;

  // Specular Blinn-Phong highlights
  vec3 H_key = normalize(L_key + V);
  float shininess = mix(95.0, 18.0, u_roughness);
  float specKey = pow(max(dot(N, H_key), 0.0), shininess);
  vec3 specColor = mix(vec3(0.18), u_baseColor, u_metallic);
  vec3 specular = specKey * C_key * specColor;

  // Architectural Rim light (Fresnel contour + back-light)
  float fresnel = 1.0 - max(dot(N, V), 0.0);
  float rimBack = pow(fresnel, 2.2) * max(dot(N, L_rim), 0.0);
  float rimContour = pow(fresnel, 3.2) * 0.45;
  vec3 rim = (rimBack + rimContour) * C_rim;

  vec3 finalColor = ambient + diffuse + specular + rim + u_emissiveColor;
  gl_FragColor = vec4(finalColor, 1.0);
}
`;

export class Mono3DViewer {
  private canvas: HTMLCanvasElement;
  private gl: WebGLRenderingContext | null = null;
  private program: WebGLProgram | null = null;
  private glbUrl: string;

  private meshes: ParsedNodeMesh[] = [];
  private state: MonoState = "idle";
  private stateChangedAt = performance.now();
  private theme: "light" | "dark" = "light";
  private gestureCallbacks: MonoGestureCallbacks = {};

  private animFrameId: number | null = null;
  private startTime = performance.now();

  // Camera orientation & controls
  private yaw = 0;
  private pitch = 0.08;
  private targetYaw = 0;
  private targetPitch = 0.08;
  private zoom = 1.0;
  private targetZoom = 1.0;
  private autoRotate = true;
  private isDragging = false;
  private pointerDown = false;
  private pointerStartX = 0;
  private pointerStartY = 0;
  private pointerDownAt = 0;
  private holdTimerId: number | null = null;
  private isHolding = false;
  private lastMouseX = 0;
  private lastMouseY = 0;

  // Cached uniform locations
  private uModelLoc: WebGLUniformLocation | null = null;
  private uViewLoc: WebGLUniformLocation | null = null;
  private uProjLoc: WebGLUniformLocation | null = null;
  private uNormalMatLoc: WebGLUniformLocation | null = null;
  private uBaseColorLoc: WebGLUniformLocation | null = null;
  private uEmissiveLoc: WebGLUniformLocation | null = null;
  private uMetallicLoc: WebGLUniformLocation | null = null;
  private uRoughnessLoc: WebGLUniformLocation | null = null;
  private uRimIntensityLoc: WebGLUniformLocation | null = null;
  private uCameraPosLoc: WebGLUniformLocation | null = null;
  private aPosLoc = -1;
  private aNormalLoc = -1;

  private isDestroyed = false;
  private isLoaded = false;

  constructor(canvas: HTMLCanvasElement, glbUrl = "/assets/AIDO-MONO.glb") {
    this.canvas = canvas;
    this.glbUrl = glbUrl;
    this.initWebGL();
    this.bindEvents();
  }

  private initWebGL(): void {
    const gl =
      this.canvas.getContext("webgl", {
        alpha: true,
        antialias: true,
        premultipliedAlpha: false,
      }) ||
      (this.canvas.getContext("experimental-webgl") as WebGLRenderingContext | null);

    if (!gl) {
      console.warn("[Mono3DViewer] WebGL is not available on this platform.");
      return;
    }
    this.gl = gl;

    // Compile Shaders
    const vs = this.compileShader(gl.VERTEX_SHADER, VS_SOURCE);
    const fs = this.compileShader(gl.FRAGMENT_SHADER, FS_SOURCE);
    if (!vs || !fs) return;

    const prog = gl.createProgram();
    if (!prog) return;
    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);

    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
      console.error("[Mono3DViewer] Program link error:", gl.getProgramInfoLog(prog));
      return;
    }
    this.program = prog;

    // Cache locations
    this.uModelLoc = gl.getUniformLocation(prog, "u_model");
    this.uViewLoc = gl.getUniformLocation(prog, "u_view");
    this.uProjLoc = gl.getUniformLocation(prog, "u_projection");
    this.uNormalMatLoc = gl.getUniformLocation(prog, "u_normalMatrix");
    this.uBaseColorLoc = gl.getUniformLocation(prog, "u_baseColor");
    this.uEmissiveLoc = gl.getUniformLocation(prog, "u_emissiveColor");
    this.uMetallicLoc = gl.getUniformLocation(prog, "u_metallic");
    this.uRoughnessLoc = gl.getUniformLocation(prog, "u_roughness");
    this.uRimIntensityLoc = gl.getUniformLocation(prog, "u_rimIntensity");
    this.uCameraPosLoc = gl.getUniformLocation(prog, "u_cameraPos");

    this.aPosLoc = gl.getAttribLocation(prog, "a_position");
    this.aNormalLoc = gl.getAttribLocation(prog, "a_normal");

    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.disable(gl.CULL_FACE); // Double-sided rendering for all facets
    gl.clearColor(0.0, 0.0, 0.0, 0.0); // Transparent canvas background
  }

  private compileShader(type: number, src: string): WebGLShader | null {
    if (!this.gl) return null;
    const shader = this.gl.createShader(type);
    if (!shader) return null;
    this.gl.shaderSource(shader, src);
    this.gl.compileShader(shader);
    if (!this.gl.getShaderParameter(shader, this.gl.COMPILE_STATUS)) {
      console.error("[Mono3DViewer] Shader compile error:", this.gl.getShaderInfoLog(shader));
      this.gl.deleteShader(shader);
      return null;
    }
    return shader;
  }

  public async load(): Promise<void> {
    if (!this.gl || !this.program) return;

    try {
      const response = await fetch(this.glbUrl);
      if (!response.ok) {
        throw new Error(`Failed to fetch ${this.glbUrl}: HTTP ${response.status}`);
      }
      const arrayBuffer = await response.arrayBuffer();
      this.parseAndUploadGLB(arrayBuffer);
      this.isLoaded = true;
      this.startLoop();
    } catch (err) {
      console.error("[Mono3DViewer] Error loading GLB model:", err);
    }
  }

  private parseAndUploadGLB(buf: ArrayBuffer): void {
    if (!this.gl) return;
    const gl = this.gl;
    const dv = new DataView(buf);

    // GLB 12-byte Header
    const magic = dv.getUint32(0, true);
    if (magic !== 0x46546c67) {
      throw new Error(`Invalid GLB magic: 0x${magic.toString(16)}`);
    }
    const version = dv.getUint32(4, true);
    if (version !== 2) {
      console.warn(`[Mono3DViewer] Unexpected glTF version: ${version}`);
    }

    // Chunk 0: JSON
    const jsonLen = dv.getUint32(12, true);
    const jsonBytes = new Uint8Array(buf, 20, jsonLen);
    const jsonStr = new TextDecoder("utf-8").decode(jsonBytes);
    const gltf = JSON.parse(jsonStr);

    // Chunk 1: BIN
    const binHeaderOffset = 20 + jsonLen;
    const binDataOffset = binHeaderOffset + 8;

    // Helper to extract typed array from accessor
    const getAccessorData = (accessorIdx: number): { data: Float32Array | Uint16Array; count: number } => {
      const acc = gltf.accessors[accessorIdx];
      const bv = gltf.bufferViews[acc.bufferView];
      const offset = binDataOffset + (bv.byteOffset || 0) + (acc.byteOffset || 0);

      if (acc.componentType === 5126) {
        // Float32 (VEC3 or VEC2)
        const components = acc.type === "VEC3" ? 3 : acc.type === "VEC2" ? 2 : 1;
        const totalFloats = acc.count * components;
        let f32: Float32Array;
        if (offset % 4 === 0) {
          f32 = new Float32Array(buf, offset, totalFloats);
        } else {
          // Copy to aligned buffer if unaligned
          const slice = new Uint8Array(buf, offset, totalFloats * 4);
          f32 = new Float32Array(slice.buffer, slice.byteOffset, totalFloats);
        }
        return { data: f32, count: acc.count };
      } else if (acc.componentType === 5123) {
        // Uint16
        let u16: Uint16Array;
        if (offset % 2 === 0) {
          u16 = new Uint16Array(buf, offset, acc.count);
        } else {
          const slice = new Uint8Array(buf, offset, acc.count * 2);
          u16 = new Uint16Array(slice.buffer, slice.byteOffset, acc.count);
        }
        return { data: u16, count: acc.count };
      }
      throw new Error(`Unsupported accessor component type: ${acc.componentType}`);
    };

    this.meshes = [];

    // Parse all nodes with meshes (Node 0: O head, Node 1: D chassis and A legs, Node 2: I core)
    for (let nodeIdx = 0; nodeIdx < gltf.nodes.length; nodeIdx++) {
      const node = gltf.nodes[nodeIdx];
      if (node.mesh === undefined) continue;

      const meshDef = gltf.meshes[node.mesh];
      const prim = meshDef.primitives[0];
      const posAccIdx = prim.attributes.POSITION;
      const normAccIdx = prim.attributes.NORMAL;
      const indicesAccIdx = prim.indices;

      const posData = getAccessorData(posAccIdx).data as Float32Array;
      const normData = getAccessorData(normAccIdx).data as Float32Array;
      const idxData = getAccessorData(indicesAccIdx).data as Uint16Array;

      // Compute centroid in post-rotated coords: (x'=x, y'=-z, z'=y)
      let cx = 0, cy = 0, cz = 0;
      const vertCount = posData.length / 3;
      for (let i = 0; i < vertCount; i++) {
        cx += posData[i * 3];
        cy += -posData[i * 3 + 2]; // Y' = -Z
        cz += posData[i * 3 + 1]; // Z' = Y
      }
      cx /= vertCount; cy /= vertCount; cz /= vertCount;

      // Upload to WebGL buffers
      const posBuf = gl.createBuffer()!;
      gl.bindBuffer(gl.ARRAY_BUFFER, posBuf);
      gl.bufferData(gl.ARRAY_BUFFER, posData, gl.STATIC_DRAW);

      const normBuf = gl.createBuffer()!;
      gl.bindBuffer(gl.ARRAY_BUFFER, normBuf);
      gl.bufferData(gl.ARRAY_BUFFER, normData, gl.STATIC_DRAW);

      const idxBuf = gl.createBuffer()!;
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, idxBuf);
      gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, idxData, gl.STATIC_DRAW);

      this.meshes.push({
        name: node.name || meshDef.name || `Node ${nodeIdx}`,
        nodeIndex: nodeIdx,
        positionBuffer: posBuf,
        normalBuffer: normBuf,
        indexBuffer: idxBuf,
        indexCount: idxData.length,
        vertexCount: vertCount,
        centroid: [cx, cy, cz],
      });
    }

    console.info(
      `[Mono3DViewer] Successfully loaded ${this.meshes.length} articulated nodes from ${this.glbUrl}:`,
      this.getNodeStats()
    );
  }

  public getNodeStats(): NodeMeshStats[] {
    return this.meshes.map((m) => ({
      name: m.name,
      nodeIndex: m.nodeIndex,
      vertices: m.vertexCount,
      indices: m.indexCount,
    }));
  }

  public setState(state: MonoState): void {
    this.state = state;
    this.stateChangedAt = performance.now();
  }

  public getState(): MonoState {
    return this.state;
  }

  public setTheme(theme: "light" | "dark"): void {
    this.theme = theme;
  }

  public getTheme(): "light" | "dark" {
    return this.theme;
  }

  public setGestureCallbacks(callbacks: MonoGestureCallbacks): void {
    this.gestureCallbacks = { ...this.gestureCallbacks, ...callbacks };
  }

  public resetCamera(): void {
    this.targetYaw = 0;
    this.targetPitch = 0.08;
    this.targetZoom = 1.0;
  }

  public setRotation(yaw: number, pitch: number): void {
    this.targetYaw = yaw;
    this.targetPitch = Math.max(-0.65, Math.min(0.75, pitch));
  }

  public setZoom(zoom: number): void {
    this.targetZoom = Math.max(0.6, Math.min(2.0, zoom));
  }

  public toggleAutoRotate(enabled?: boolean): boolean {
    if (enabled !== undefined) {
      this.autoRotate = enabled;
    } else {
      this.autoRotate = !this.autoRotate;
    }
    return this.autoRotate;
  }

  public isAutoRotate(): boolean {
    return this.autoRotate;
  }

  private clearHoldTimer(): void {
    if (this.holdTimerId !== null) {
      window.clearTimeout(this.holdTimerId);
      this.holdTimerId = null;
    }
  }

  private bindEvents(): void {
    // Prevent middle-click auto-scroll on the canvas
    this.canvas.addEventListener("mousedown", (e) => {
      if (e.button === 1) {
        e.preventDefault();
      }
    });

    this.canvas.addEventListener("auxclick", (e) => {
      if (e.button === 1) {
        e.preventDefault();
        this.gestureCallbacks.onMiddleClick?.(e);
      }
    });

    // Right-click custom context menu
    this.canvas.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      this.gestureCallbacks.onRightClick?.(e);
    });

    this.canvas.addEventListener("pointerdown", (e) => {
      // Middle mouse button handled via auxclick/mousedown
      if (e.button === 1) {
        e.preventDefault();
        return;
      }
      if (e.button !== 0) return;

      this.pointerDown = true;
      this.isDragging = false;
      this.isHolding = false;
      this.pointerStartX = e.clientX;
      this.pointerStartY = e.clientY;
      this.lastMouseX = e.clientX;
      this.lastMouseY = e.clientY;
      this.pointerDownAt = performance.now();

      try {
        this.canvas.setPointerCapture(e.pointerId);
      } catch (_) {}

      this.clearHoldTimer();
      this.holdTimerId = window.setTimeout(() => {
        this.holdTimerId = null;
        if (this.pointerDown && !this.isDragging) {
          this.isHolding = true;
          this.gestureCallbacks.onHoldStart?.(e);
        }
      }, 380);
    });

    this.canvas.addEventListener("pointermove", (e) => {
      if (!this.pointerDown) return;

      const totalDx = e.clientX - this.pointerStartX;
      const totalDy = e.clientY - this.pointerStartY;
      const dist = Math.hypot(totalDx, totalDy);

      if (!this.isDragging && dist > 6) {
        this.isDragging = true;
        this.clearHoldTimer();
        if (this.isHolding) {
          this.isHolding = false;
          const durationMs = Math.max(0, performance.now() - this.pointerDownAt);
          this.gestureCallbacks.onHoldEnd?.(e, durationMs);
        }
      }

      if (this.isDragging) {
        const dx = e.clientX - this.lastMouseX;
        const dy = e.clientY - this.lastMouseY;
        this.lastMouseX = e.clientX;
        this.lastMouseY = e.clientY;

        this.targetYaw += dx * 0.009;
        this.targetPitch = Math.max(-0.65, Math.min(0.75, this.targetPitch + dy * 0.009));
      }
    });

    const stopPointer = (e: PointerEvent, isCancel: boolean) => {
      if (!this.pointerDown) return;
      this.pointerDown = false;
      this.clearHoldTimer();

      const durationMs = Math.max(0, performance.now() - this.pointerDownAt);
      const totalDist = Math.hypot(
        e.clientX - this.pointerStartX,
        e.clientY - this.pointerStartY
      );

      if (this.isHolding) {
        this.isHolding = false;
        this.gestureCallbacks.onHoldEnd?.(e, durationMs);
      } else if (!isCancel && !this.isDragging && totalDist <= 6 && durationMs < 380) {
        this.gestureCallbacks.onShortClick?.(e);
      }

      this.isDragging = false;
      try {
        this.canvas.releasePointerCapture(e.pointerId);
      } catch (_) {}
    };

    this.canvas.addEventListener("pointerup", (e) => stopPointer(e, false));
    this.canvas.addEventListener("pointercancel", (e) => stopPointer(e, true));

    this.canvas.addEventListener("wheel", (e) => {
      e.preventDefault();
      const delta = e.deltaY * 0.0015;
      this.targetZoom = Math.max(0.6, Math.min(2.0, this.targetZoom - delta));
    }, { passive: false });

    this.canvas.addEventListener("dblclick", () => {
      this.resetCamera();
    });
  }

  private startLoop(): void {
    const render = () => {
      if (this.isDestroyed) return;
      this.render();
      this.animFrameId = requestAnimationFrame(render);
    };
    this.animFrameId = requestAnimationFrame(render);
  }

  private render(): void {
    if (!this.gl || !this.program || !this.isLoaded) return;
    const gl = this.gl;

    // Handle high-DPI canvas resizing
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const displayWidth = Math.round(this.canvas.clientWidth * dpr);
    const displayHeight = Math.round(this.canvas.clientHeight * dpr);

    if (this.canvas.width !== displayWidth || this.canvas.height !== displayHeight) {
      this.canvas.width = displayWidth || 280;
      this.canvas.height = displayHeight || 280;
    }

    gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    gl.useProgram(this.program);

    // Smooth camera inertia & auto-rotation (paused during sleep or dragging)
    if (this.autoRotate && !this.isDragging && this.state !== "sleep") {
      this.targetYaw += 0.003;
    }
    this.yaw += (this.targetYaw - this.yaw) * 0.12;
    this.pitch += (this.targetPitch - this.pitch) * 0.12;
    this.zoom += (this.targetZoom - this.zoom) * 0.12;

    const aspect = gl.drawingBufferWidth / (gl.drawingBufferHeight || 1);
    const projMat = mat4Perspective((38 * Math.PI) / 180, aspect, 0.1, 20.0);

    // Camera sits in front of character at distance 3.3 / zoom
    const camDist = 3.3 / this.zoom;
    const eye: [number, number, number] = [0, 0.05, camDist];
    const center: [number, number, number] = [0, 0, 0];
    const up: [number, number, number] = [0, 1, 0];
    const viewMat = mat4LookAt(eye, center, up);

    const isDark = this.theme === "dark";
    const rimIntensity = isDark ? 1.15 : 0.92;

    gl.uniformMatrix4fv(this.uProjLoc, false, projMat);
    gl.uniformMatrix4fv(this.uViewLoc, false, viewMat);
    gl.uniform1f(this.uRimIntensityLoc, rimIntensity);
    gl.uniform3f(this.uCameraPosLoc, eye[0], eye[1], eye[2]);

    // Elapsed animation time in seconds
    const now = performance.now();
    const t = (now - this.startTime) * 0.001;
    const stateElapsed = Math.max(0, (now - this.stateChangedAt) * 0.001);

    // Base character rotation & centering
    // The character is centered around Y' = 1.0 (from 0 to 2.0).
    // Center offset shifts Y by -1.0 so centroid is at (0, 0, 0).
    const rotPitch = mat4RotateX(this.pitch);
    const rotYaw = mat4RotateY(this.yaw);
    const rotCam = mat4Identity();
    mat4Multiply(rotCam, rotPitch, rotYaw);

    const centerOffset = mat4Translate(0, -1.0, 0);

    // glTF node rotation: +90 deg around X ([0.7071068, 0, 0, 0.7071068])
    // Maps raw (x, y, z) into Y-up (x, -z, y)
    const gltfNodeRot = mat4RotateX(Math.PI / 2);

    // Pure Monochrome Material Palette (Light Mode #FFFFFF vs Dark Mode #000000)
    const headBodyColor: [number, number, number] = isDark
      ? [0.18, 0.18, 0.20] // Sculpted satin titanium on #000000
      : [0.08, 0.08, 0.09]; // Deep architectural obsidian on #FFFFFF
    const chassisBodyColor: [number, number, number] = isDark
      ? [0.15, 0.15, 0.17]
      : [0.06, 0.06, 0.07];

    // Compute independent 3-node articulation
    for (const mesh of this.meshes) {
      // Articulation matrix in Y-up character space
      let artMat = mat4Identity();

      let baseColor: [number, number, number] = headBodyColor;
      let emissiveColor: [number, number, number] = [0.0, 0.0, 0.0];
      let metallic = isDark ? 0.22 : 0.14;
      let roughness = isDark ? 0.36 : 0.40;

      if (mesh.nodeIndex === 0) {
        // --- NODE 0: MONO · O head ---
        // Neck connection is at Y' ~ 1.30
        const neckY = 1.30;
        let hoverY = 0;
        let headPitch = 0;
        let headYaw = 0;
        let headRoll = 0;

        if (this.state === "sleep") {
          // Lowered head, tilted forward/down, very slow deep breathing
          hoverY = -0.06 + Math.sin(t * 1.1) * 0.012;
          headPitch = 0.28 + Math.sin(t * 1.1) * 0.015;
          headYaw = 0;
          headRoll = 0;
        } else if (this.state === "wake") {
          // Over the first 1.0s since stateChangedAt, rise along an arc from lowered (-0.06, pitch 0.25)
          // up to upright (+0.03, pitch -0.05) with a gentle welcoming nod
          const p = Math.min(1.0, stateElapsed / 1.0);
          const easeOut = 1.0 - Math.pow(1.0 - p, 3);
          const welcomeNod = Math.sin(p * Math.PI * 2.0) * 0.06 * (1.0 - p * 0.5);
          hoverY = -0.06 + (0.03 - -0.06) * easeOut + Math.sin(t * 2.4) * 0.015 * easeOut;
          headPitch = 0.25 + (-0.05 - 0.25) * easeOut + welcomeNod;
          headYaw = Math.sin(p * Math.PI) * 0.04;
        } else if (this.state === "idle") {
          // Gentle vertical hover, subtle attentive head micro-movement
          hoverY = Math.sin(t * 2.2) * 0.035;
          headPitch = Math.sin(t * 2.2) * 0.02;
          headYaw = Math.sin(t * 1.1) * 0.025;
        } else if (this.state === "listen" || this.state === "listening") {
          // Attentive head tilt, responsive vertical hover
          hoverY = Math.sin(t * 3.4) * 0.03;
          headRoll = 0.11;
          headPitch = -0.06;
          headYaw = Math.sin(t * 1.7) * 0.025;
        } else if (this.state === "think" || this.state === "thinking") {
          // Thoughtful scanning orbit
          hoverY = Math.sin(t * 3.4) * 0.03;
          headYaw = Math.sin(t * 3.8) * 0.20;
          headPitch = Math.cos(t * 1.9) * 0.05;
        } else if (this.state === "work") {
          // Focused, purposeful double-time articulation
          hoverY = Math.sin(t * 5.2) * 0.026;
          headYaw = Math.sin(t * 5.2) * 0.12;
          headPitch = Math.abs(Math.sin(t * 5.2)) * 0.08;
          headRoll = Math.cos(t * 2.6) * 0.03;
        } else if (this.state === "speak" || this.state === "speaking") {
          // Natural speech cadence nod
          hoverY = Math.sin(t * 7.5) * 0.032;
          headPitch = Math.sin(t * 7.5) * 0.075;
          headRoll = Math.sin(t * 3.75) * 0.02;
        } else if (this.state === "success") {
          // Affirmative double-nod of the O head + joyful upward lift
          const decay = Math.exp(-stateElapsed * 0.8);
          hoverY = 0.045 * decay + Math.sin(t * 2.4) * 0.025;
          headPitch = Math.sin(stateElapsed * 9.0) * 0.14 * decay;
          headRoll = Math.sin(stateElapsed * 4.5) * 0.03 * decay;
        } else if (this.state === "error") {
          // Clear horizontal head-shake + slight defensive tilt
          const decay = Math.exp(-stateElapsed * 0.9);
          hoverY = -0.015 * decay + Math.sin(t * 2.2) * 0.02;
          headYaw = Math.sin(stateElapsed * 16.0) * 0.22 * decay;
          headRoll = -0.06 * decay;
          headPitch = 0.04 * decay;
        }

        const tNeck = mat4Translate(0, neckY, 0);
        const tNegNeck = mat4Translate(0, -neckY, 0);
        const rHead = mat4Identity();
        const rP = mat4RotateX(headPitch);
        const rY = mat4RotateY(headYaw);
        const rR = mat4RotateZ(headRoll);
        mat4Multiply(rHead, rP, rY);
        mat4Multiply(rHead, rHead, rR);

        const tHover = mat4Translate(0, hoverY, 0);

        // artMat = tHover * tNeck * rHead * tNegNeck
        mat4Multiply(artMat, tNeck, rHead);
        mat4Multiply(artMat, artMat, tNegNeck);
        mat4Multiply(artMat, tHover, artMat);

        baseColor = headBodyColor;
      } else if (mesh.nodeIndex === 1) {
        // --- NODE 1: MONO · D chassis and A legs ---
        let breath = 1.0 + Math.sin(t * 2.0) * 0.005;
        if (this.state === "sleep") {
          breath = 1.0 + Math.sin(t * 1.1) * 0.003;
        } else if (this.state === "work") {
          breath = 1.0 + Math.sin(t * 5.2) * 0.008;
        } else if (this.state === "wake") {
          const p = Math.min(1.0, stateElapsed / 1.0);
          breath = 1.0 + Math.sin(p * Math.PI) * 0.01;
        }
        artMat = mat4Scale(breath, breath, breath);

        baseColor = chassisBodyColor;
        metallic = isDark ? 0.18 : 0.10;
        roughness = isDark ? 0.40 : 0.44;
      } else if (mesh.nodeIndex === 2) {
        // --- NODE 2: MONO · I core (Strictly Monochrome Pure White / Silver Core) ---
        // Core centroid is at (0, 0.897, 0)
        const coreY = 0.897;
        let pulseScale = 1.0;
        let glowIntensity = 0.45;

        if (this.state === "sleep") {
          // Dimmed, calm slow pulse
          pulseScale = 0.92 + Math.sin(t * 1.1) * 0.02;
          baseColor = [0.58, 0.58, 0.60];
          glowIntensity = 0.08 + Math.sin(t * 1.1) * 0.03;
        } else if (this.state === "wake") {
          // Core flares brightly as character wakes
          const p = Math.min(1.0, stateElapsed / 1.0);
          const flare = Math.sin(p * Math.PI) * 0.14;
          pulseScale = 0.95 + 0.12 * p + flare;
          baseColor = [0.98, 0.98, 1.0];
          glowIntensity = 0.55 + flare * 2.2;
        } else if (this.state === "idle") {
          // Crisp core breathing
          pulseScale = 1.0 + Math.sin(t * 2.5) * 0.045;
          baseColor = [0.95, 0.95, 0.97];
          glowIntensity = 0.42 + Math.sin(t * 2.5) * 0.08;
        } else if (this.state === "listen" || this.state === "listening") {
          // High-contrast bright white/silver pulsing core
          pulseScale = 1.12 + Math.sin(t * 5.0) * 0.07;
          baseColor = [1.0, 1.0, 1.0];
          glowIntensity = 0.72 + Math.sin(t * 5.0) * 0.18;
        } else if (this.state === "think" || this.state === "thinking") {
          // Rhythmic core oscillation
          pulseScale = 1.08 + Math.sin(t * 6.8) * 0.09;
          baseColor = [0.96, 0.96, 0.98];
          glowIntensity = 0.62 + Math.sin(t * 6.8) * 0.22;
        } else if (this.state === "work") {
          // Rapid precision pulse
          pulseScale = 1.14 + Math.sin(t * 9.0) * 0.08;
          baseColor = [1.0, 1.0, 1.0];
          glowIntensity = 0.76 + Math.sin(t * 9.0) * 0.20;
        } else if (this.state === "speak" || this.state === "speaking") {
          // Natural speech cadence pulse
          pulseScale = 1.08 + Math.sin(t * 7.5) * 0.08;
          baseColor = [0.98, 0.98, 1.0];
          glowIntensity = 0.65 + Math.sin(t * 7.5) * 0.18;
        } else if (this.state === "success") {
          // Bright core illumination
          const decay = Math.exp(-stateElapsed * 0.8);
          pulseScale = 1.05 + 0.14 * decay;
          baseColor = [1.0, 1.0, 1.0];
          glowIntensity = 0.50 + 0.45 * decay;
        } else if (this.state === "error") {
          // Sharp high-contrast core pulse
          const decay = Math.exp(-stateElapsed * 0.9);
          const strobe = Math.abs(Math.sin(stateElapsed * 16.0)) * decay;
          pulseScale = 1.02 + 0.16 * strobe;
          baseColor = [0.96, 0.96, 0.98];
          glowIntensity = 0.35 + 0.60 * strobe;
        }

        emissiveColor = [glowIntensity, glowIntensity, glowIntensity];

        const tCore = mat4Translate(0, coreY, 0);
        const tNegCore = mat4Translate(0, -coreY, 0);
        const sCore = mat4Scale(pulseScale, pulseScale, pulseScale);

        mat4Multiply(artMat, tCore, sCore);
        mat4Multiply(artMat, artMat, tNegCore);

        metallic = 0.30;
        roughness = 0.20;
      }

      // Final Model Matrix for this node:
      // model = rotCam * centerOffset * artMat * gltfNodeRot
      const modelMat = mat4Identity();
      mat4Multiply(modelMat, centerOffset, artMat);
      mat4Multiply(modelMat, modelMat, gltfNodeRot);
      mat4Multiply(modelMat, rotCam, modelMat);

      // Normal Matrix: Inverse transpose of upper 3x3 of modelMat
      const normalMat = new Float32Array(9);
      mat3NormalFromMat4(normalMat, modelMat);

      gl.uniformMatrix4fv(this.uModelLoc, false, modelMat);
      gl.uniformMatrix3fv(this.uNormalMatLoc, false, normalMat);
      gl.uniform3fv(this.uBaseColorLoc, baseColor);
      gl.uniform3fv(this.uEmissiveLoc, emissiveColor);
      gl.uniform1f(this.uMetallicLoc, metallic);
      gl.uniform1f(this.uRoughnessLoc, roughness);

      // Bind attributes and draw
      gl.bindBuffer(gl.ARRAY_BUFFER, mesh.positionBuffer);
      gl.enableVertexAttribArray(this.aPosLoc);
      gl.vertexAttribPointer(this.aPosLoc, 3, gl.FLOAT, false, 0, 0);

      gl.bindBuffer(gl.ARRAY_BUFFER, mesh.normalBuffer);
      gl.enableVertexAttribArray(this.aNormalLoc);
      gl.vertexAttribPointer(this.aNormalLoc, 3, gl.FLOAT, false, 0, 0);

      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, mesh.indexBuffer);
      gl.drawElements(gl.TRIANGLES, mesh.indexCount, gl.UNSIGNED_SHORT, 0);
    }
  }

  public destroy(): void {
    this.isDestroyed = true;
    this.clearHoldTimer();
    if (this.animFrameId !== null) {
      cancelAnimationFrame(this.animFrameId);
      this.animFrameId = null;
    }
    if (this.gl) {
      for (const m of this.meshes) {
        this.gl.deleteBuffer(m.positionBuffer);
        this.gl.deleteBuffer(m.normalBuffer);
        this.gl.deleteBuffer(m.indexBuffer);
      }
      if (this.program) {
        this.gl.deleteProgram(this.program);
      }
    }
    this.meshes = [];
  }
}
