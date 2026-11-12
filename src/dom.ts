type Child = Node | string;

type ElProps = {
  className?: string;
  text?: string;
  attrs?: Record<string, string>;
};

/**
 * Erzeugt ein HTML-Element, ohne Markup zu parsen.
 * Text landet in Textknoten, Attribute nur über setAttribute.
 */
export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  props?: ElProps,
  ...children: Child[]
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (props?.className) node.className = props.className;
  if (props?.attrs) {
    for (const [name, value] of Object.entries(props.attrs)) {
      node.setAttribute(name, value);
    }
  }
  if (props?.text !== undefined) node.append(document.createTextNode(props.text));
  for (const child of children) {
    node.append(typeof child === "string" ? document.createTextNode(child) : child);
  }
  return node;
}

const SVG_NS = "http://www.w3.org/2000/svg";

export function svg<K extends keyof SVGElementTagNameMap>(
  tag: K,
  attrs?: Record<string, string>,
  ...children: Node[]
): SVGElementTagNameMap[K] {
  const node = document.createElementNS(SVG_NS, tag);
  if (attrs) {
    for (const [name, value] of Object.entries(attrs)) node.setAttribute(name, value);
  }
  for (const child of children) node.append(child);
  return node;
}
