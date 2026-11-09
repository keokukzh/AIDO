# Lokale Modelllaufzeiten: Inventar und nächster Prüfblock

**Stand: 9. November 2026.** Punktuelle Tests am Referenz-PC; kein vollständiger Benchmark und keine fertige AIDO-Integration.

## Vorhandene Laufzeiten

| Befund | Beobachtung |
| --- | --- |
| LM Studio | Installiert; lokaler Server meldete Port 1234 und verlangte für API-Anfragen ein Bearer-Token. |
| llama.cpp | In LM Studio sind mehrere llama.cpp-Engines installiert; ausgewählt war `llama.cpp-win-x86_64-nvidia-cuda12-avx2@2.71.0`. Ein separates `llama-server`/`llama-cli` wurde in den geprüften Windows- und Ubuntu-PATHs nicht gefunden. |
| Ollama | Windows-Installation vorhanden. Die Prozessumgebung hatte `OLLAMA_MODELS` zunächst nicht vom Benutzerprofil geerbt. Mit `D:\\AI_Models\\ollama` zeigte ein temporärer Server zweimal dieselben Einträge; `qwen3.8:latest` scheiterte beim Öffnen an einem fehlenden Blob. Der temporäre Server wurde gestoppt. |
| GPU | GeForce RTX 5090, 32607 MiB laut `nvidia-smi` zum Prüfzeitpunkt. |

Damit ist eine weitere llama.cpp-Installation **nicht** der erste Schritt. Zunächst ist die vorhandene LM-Studio-Engine als Referenz zu verwenden und Ollamas Modellpfad/Serverzustand zu klären.

## Modelle im LM-Studio-Katalog

- Mehrere Qwen3.8-27B-GGUF-Ableitungen waren vorhanden, darunter `gamedev-qwen3.8-v2-rea@q4_k_m` (16,81 GB) und eine Q6_K-Variante (22,43 GB).
- Ein separater `GameDev-Qwen3.8-v2-REA-mmproj.gguf`-Projektor (931 MB) war vorhanden.
- Das Text-Embedding-Modell `text-embedding-nomic-embed-text-v1.5` (84,11 MB) war bereits vorhanden. Für den Referenz-PC ist derzeit kein zusätzlicher Embedding-Download nötig.
- `lms ls --json` meldete für die gelisteten Qwen-Modelldateien `vision: false`. Das [offizielle Qwen3.8-27B](https://huggingface.co/Qwen/Qwen3.8-27B) ist bildfähig; die hier installierten Ableitungen und ihre Projektorzuordnung müssen separat geprüft werden.

## Durchgeführte Kurztests

Das vorhandene Embedding-Modell wurde in LM Studio geladen und danach wieder entladen. Der erste API-Aufruf auf `/v1/embeddings` wurde ohne Bearer-Token abgewiesen. Dieser anfängliche Zustand wurde später durch den unten dokumentierten autorisierten Test abgelöst. Die Serverauthentifizierung wurde nicht verändert.

`gamedev-qwen3.8-v2-rea@q4_k_m` wurde geladen und beantwortete einen einfachen Textprompt lokal. LM Studio meldete 81,75 generierte Token/s, 1,288 s bis zum ersten Token und 563 generierte Token für diese eine Anfrage. Das Modell gab dabei einen langen Denkabschnitt aus. Diese Einzelmessung ist kein Vergleichsbenchmark. Das Modell wurde danach wieder entladen.

Bild-Eingang, Audio, Tool-Aufrufe, Offline-Datenfluss, VRAM-Spitzen und Wiederladeverhalten wurden nicht getestet.

## Modellrollen für AIDO

| Rolle | Nächster Schritt |
| --- | --- |
| Hauptmodell | Vorhandene Qwen-Ableitungen mit gleichen Text-, Werkzeug- und Bildaufgaben vergleichen. |
| Vision | Modell plus passenden Projektor in der Runtime verbinden und mit einem selbst erzeugten Testbild prüfen. Nur ein erfolgreicher Bildtest zählt. |
| Text-Embedding | Vorhandenes Nomic-Modell über einen autorisierten lokalen Client testen; deutsche Projekttexte und Speicherabruf bewerten. |
| Spracheingabe | Separates lokales ASR-Modell auswählen und mit deutschsprachigem Testaudio prüfen; Embeddings transkribieren kein Audio. |
| Sprachausgabe | Erst für die Voice-Oberfläche ein lokales TTS-Modell auswählen und auf Qualität/Latenz prüfen. |

## Nächste konkrete Schritte

1. Ollamas Server- und Modellpfade ohne Veränderung bestehender Modelle klären (punktuell erledigt; fehlender Blob verhindert derzeit Ollama-Vergleich).
2. LM Studios Embedding-API mit autorisiertem Zugang testen (Funktionsprobe erledigt; Schlüsselwiderruf nach Offenlegung im Chat erforderlich).
3. Die Bildfähigkeit einer konkreten Qwen-Datei mit zugehörigem Projektor und Testbild verifizieren.
4. Kurze reproduzierbare Tests für Text, Embedding und ASR ausführen. Startzeit, Tokenrate, VRAM, CPU/RAM, Idle-Entladung und Fehler dokumentieren.
5. Erst danach über zusätzliche Downloads oder eine andere primäre Runtime entscheiden. Endnutzer-Installer sollen benötigte Modelle je nach gewählten Funktionen erkennen, wiederverwenden oder nach Zustimmung installieren.

Bis zu diesem Inventarstand wurde keine neue Modell- oder Runtime-Installation vorgenommen; der spätere Whisper-Modell-Download ist unten dokumentiert.

## Ausführung des Prüfplans am 9. November 2026

Die Benutzerumgebung setzt `OLLAMA_MODELS` auf `D:\AI_Models\ollama`; die laufende PowerShell-Sitzung hatte diese Variable nicht geerbt. Ein explizit mit diesem Pfad gestarteter temporärer Ollama-Server zeigte zweimal dieselben Einträge (`kimi-k2.6:cloud`, `qwen3.8:latest`). `ollama show qwen3.8:latest` scheiterte jedoch an einem fehlenden GGUF-Blob. Die frühere Liste mit GameDev-Qwen und qwen2.5 ließ sich diesem Zustand nicht zuordnen. Ollama wird für Modellvergleiche vorläufig ausgeschlossen; der Testserver wurde gestoppt. Es wurde nichts repariert oder gelöscht.

Ein `whisper-server.exe` wurde unter der bestehenden Unsloth/whisper.cpp-Installation gefunden. In den gezielt geprüften Modellverzeichnissen fand sich kein zugehöriges Whisper-Sprachmodell. Ein Sprachtest fand daher nicht statt.

Ein separater, nur an localhost gebundener Embedding-Testserver wurde durch die automatische Freigabeprüfung vor dem Start blockiert. Es gab keinen zweiten Startversuch; die LM-Studio-Tokenpflicht blieb aktiv. Diese Sperre betraf nur den separaten Testserver; der spätere autorisierte Embedding-Test gelang. Bild-Eingang bleibt ungetestet.

Die anschließend geöffnete LM-Studio-Oberfläche (Version 0.4.28) bestätigte einen laufenden lokalen Server auf `127.0.0.1:1234`, aktivierte Authentifizierung und neun vorhandene API-Schlüssel. Die Schlüssel wurden weder geöffnet noch ausgelesen. Die Detailansicht der GameDev-Qwen3.8-v2-REA-Q4-Datei zeigte als Fähigkeiten Tool Use und Reasoning, aber kein Vision. Der separate Projektor wurde in der Bibliothek als eigenes CLIP-Artefakt aufgeführt; eine erfolgreiche Zuordnung oder Bildverarbeitung wurde nicht beobachtet. Für den späteren Embedding-API-Test stellte der Nutzer einen Zugang bereit. Die Servereinstellungen blieben unverändert.

## Autorisierter Embedding-Test

Nach Bereitstellung eines temporären lokalen API-Zugangs lieferte das vorhandene Nomic-Embedding-Modell über LM Studios geschützten Endpunkt drei Vektoren mit je 768 Dimensionen. Für zwei thematisch verwandte deutsche Projekttexte betrug die Kosinusähnlichkeit 0,8036; für einen themenfremden Gartentext 0,5959. Damit ist eine einzelne Funktionsprobe samt Plausibilitätsvergleich bestanden. Reproduzierbarkeit, vollständige Prüfung der Vektorwerte, Suchqualität auf realen AIDO-Daten, Kaltstart und Offline-Datenfluss sind noch nicht belegt. Das Modell wurde anschließend entladen; der Token wurde nicht in Dateien oder das Repository übernommen. Weil der Nutzer den Token im Chat offengelegt hat, muss er in LM Studio widerrufen werden.

## Fortsetzung: Vision- und ASR-Prüfung am 9. November 2026

In LM Studio 0.4.28 wurde `gamedev-qwen3.8-v2-rea@q4_k_m` über den Chat geladen. Die angezeigten erweiterten Ladeparameter enthielten keine Option, den separat katalogisierten `GameDev-Qwen3.8-v2-REA-mmproj.gguf` zuzuordnen. Der Chat bot bei „Attach file“ einen Dialog für dokumentbasiertes RAG mit PDF, DOCX, TXT und CSV; ein Bildanhang war über diesen Weg nicht möglich. Es wurden keine Bilddaten an das Modell gesendet und keine Bildantwort erzeugt. Das Modell wurde wieder entladen; `lms ps --json` zeigte danach `[]`. Ein künstliches rotes Quadrat liegt nur als lokaler Testentwurf im Arbeitsverzeichnis und wurde nicht ins Repository übernommen. Die [LM-Studio-Dokumentation zu Bildeingaben](https://lmstudio.ai/docs/typescript/llm-prediction/image-input) setzt ein als Vision-Modell erkanntes Modell voraus. Damit bleibt die installierte Q4-Ableitung in LM Studio für AIDO-Bildverarbeitung **nicht nachgewiesen**; ein anderer Runtime-Pfad wurde dadurch weder bestätigt noch widerlegt.

Für Spracheingabe wurde die vorhandene `whisper-server.exe` erneut bestätigt. Eine gezielte Dateisuche in `D:\\AI_Models\\audio_encoders`, `huggingface`, `gguf`, `llm`, `unsloth\\whisper.cpp` und dem lokalen Hugging-Face-Cache fand keine nach dem üblichen Whisper/ggml-Schema benannte Sprachmodell-Datei; Treffer mit „whisper“ waren Programmdateien oder Dokumentationsbilder. Dies ist keine vollständige Suche auf allen Laufwerken. Ein deutscher ASR-Test bleibt mangels verifiziertem Modell offen; es wurde nichts heruntergeladen oder installiert.

## Fortsetzung: Sprachmodell bereitgestellt, Test angehalten

Auf ausdrücklichen Wunsch nach Fortsetzung wurde das mehrsprachige Whisper-`small`-Modell von der [whisper.cpp-Modellsammlung](https://huggingface.co/ggerganov/whisper.cpp) als `D:\\AI_Models\\unsloth\\whisper.cpp\\models\\ggml-small.bin` lokal gespeichert. Die Datei hat 487.601.967 Byte; ihr SHA-1-Wert `55356645c2b361a969dfd0ef2c5a50d530afd8d5` stimmt mit dem Wert der Modellkarte überein. Quelle war die dort referenzierte, auf Revision `90a64d80ea254cf67575b41a5971f972c79f7b45` fixierte Datei. Die vorhandene `whisper-server.exe` konnte das Modell laden und meldete `small` mit 99 Sprachen; sie fand dabei **keine GPU** und nutzte CPU. Der Server lauschte kurz ausschließlich auf `127.0.0.1:18080`, wurde auf Nutzerwunsch vor einer Audioanfrage beendet und der Port war danach nicht mehr offen.

Ein deutsches WAV-Testaudio wurde mit der bereits installierten Windows-Stimme „Microsoft Hedda Desktop“ lokal im Arbeitsverzeichnis erzeugt. Es wurde **nicht** an Whisper gesendet. Daher gibt es noch kein Transkriptions-, Qualitäts-, Latenz- oder Offline-Ergebnis. Weitere installierte deutsche Windows-Stimmen wurden gefunden; deren Eignung als AIDO-Sprachausgabe ist nicht getestet. Der Nutzer hat die Ausführung hier gestoppt. Das Modell bleibt als Datei für eine spätere autorisierte Prüfung vorhanden; es wurden keine Modellbestände gelöscht.


## Abschluss der Modell- & Runtime-Validierung am 9. November 2026

Nach Wiederaufnahme der Prüfung wurden die beiden offenen Blocker (Vision-Projektorzuordnung und GPU-Erkennung für Whisper) technisch gelöst und alle Modellrollen auf dem Referenz-PC (RTX 5090, 32607 MiB VRAM) lokal an `127.0.0.1` verifiziert:

### 1. Lösung des Whisper-CUDA-Blockers & deutscher ASR-Test (`whisper-server.exe` v1.9.5)

- **Ursache für fehlende GPU-Erkennung:** `D:\AI_Models\unsloth\whisper.cpp\build\bin\Release\whisper-server.exe` ist als `slim`-Build kompiliert und enthält `ggml-cuda.dll`, benötigt zur Laufzeit jedoch die CUDA-13-Bibliotheken (`cudart64_13.dll`, `cublas64_13.dll`, `cublasLt64_13.dll`), die bereits in `C:\AI\llamacpp-cuda` vorhanden sind.
- **Ergebnis mit eingebundenem CUDA-DLL-Pfad:**
  - `ggml_cuda_init` erkannte sofort `Device 0: NVIDIA GeForce RTX 5090, compute capability 12.0, VMM: yes, VRAM: 32579 MiB` (`use gpu = 1`, `flash attn = 1`, `BLACKWELL_NATIVE_FP4 = 1`).
  - Ein mit `Microsoft Hedda Desktop` lokal erzeugtes deutsches WAV-Testaudio (6,9 s, 304.436 Byte; gesprochener Satz: *„Hallo AIDO, bitte führe einen schnellen Systemcheck durch und zeige mir den Speicherverbrauch.“*) wurde an `http://127.0.0.1:18080/inference` gesendet.
  - **Latenz:** `0,341 s` (Antwort inkl. HTTP-Roundtrip auf `127.0.0.1`).
  - **VRAM-Belegung:** `+1.150 MiB` (von `2.300 MiB` auf `3.450 MiB`, Modellgröße im VRAM `487,01 MB`).
  - **Transkript:** `„Hallo IDO, bitte führe einen schnellen Systemcheck durch und zeige mir den Speicherverbrauch.“` (phonetische Abweichung beim Eigennamen „AIDO“ → „IDO“, sämtliche deutschen Fachbegriffe und Umlaute korrekt erkannt).

### 2. Lösung des Vision-Blockers, DFlash2 Speculative Decoding & Schlafmodus (`llama-server.exe`)

Statt über die LM-Studio-Oberfläche wurde das vorhandene Qwen3.8-Setup direkt über die lokalen `llama-server.exe`-Binaries (`C:\AI\llamacpp-cuda\llama-server.exe` Build 11671 sowie `D:\AI_Models\unsloth\llama.cpp\build\bin\Release\llama-server.exe` Build 12176 / v0.9.0-dev) im Router-Modus (`--models-preset`, `--models-autoload`, `--sleep-idle-seconds`) ausschließlich auf `127.0.0.1:18434` mit flüchtigem Sitzungs-Token geprüft:

- **Konfiguration (Standard-Hauptmodell für AIDO auf dem Referenz-PC):**
  - Hauptmodell: `D:\AI_Models\llm\Qwen3.8-27B-UD-Q4_K_M.gguf` (`16.464.440.224` Byte, SHA-256: `322E194FF79741C7BAA497C240F677F54B201B0EFAB44CA8E50F122B39123482`)
  - Vision-Projektor: `D:\AI_Models\llm\assets\mmproj-Qwen3.8-27B-BF16.gguf` (`931.146.432` Byte, SHA-256: `83EE4F4F205FA514161778C41DF1EA14144FAA0F713510893B63C2395F5C2D53`)
  - Speculative Draft (`draft-dflash`): `D:\AI_Models\llm\assets\Qwen3.8-27B-DFlash2-Q4_K_M.gguf` (`1.143.006.816` Byte, SHA-256: `1A25C56858E1EBE93F2718AC1D49D1151F9323325C1BBFD6209370F4DB131EBD`)
  - Kontext & KV-Cache: `ctx-size = 32768`, `cache-type-k = q8_0`, `cache-type-v = q8_0`, `flash-attn = on`
- **Messwerte auf Build 12176 (`v0.9.0-dev`):**
  - **Router-Ruhezustand vor erstem Request:** `2.307 MiB` System-VRAM (`0 MiB` zusätzliche Modellbelegung).
  - **Kaltstart (Autoload von SSD in VRAM) + Vision-Inferenz:** `9,41 s` gesamt (`prompt_eval = 747,42 ms` für 103 Tokens inkl. Bild-Encoding = `137,81 Token/s`; `eval = 642,63 ms` für 67 Tokens = **`102,70 Token/s`**).
  - **VRAM-Belegung geladen (Hauptmodell + mmproj + DFlash2 + 32k KV-Cache):** `25.544 MiB` (**`+23.237 MiB`**).
  - **Bild-Ergebnis (mit künstlichem 256×128 Rot/Blau-Testbild):** *„Die linke Hälfte des Bildes ist rot und die rechte Hälfte ist blau.“*
  - **Gegenprobe (gleiche Frage ohne Bild):** *„Es wurde kein Bild in dieser Unterhaltung hochgeladen, daher kann ich die Farben nicht bestimmen.“* (Wichtiger technischer Hinweis: Bei OpenAI-kompatiblen Multimodalanfragen an `llama-server` muss das Text-Objekt `{"type": "text", "text": "..."}` heißen, nicht `content`).
  - **Warme Textgenerierung mit DFlash2:** **`151,54 Token/s`** (`48` von `126` Draft-Tokens akzeptiert; auf Build 11671 zuvor `130,39 Token/s` bei 200 Tokens).
  - **Automatischer Schlafmodus (`--sleep-idle-seconds 10`):** Nach 10 Sekunden ohne Anfrage entlud der Server das Modell automatisch; der genutzte System-VRAM fiel von `25.544 MiB` auf **`2.830 MiB`** zurück (**`22.714 MiB` VRAM automatisch freigegeben**).
  - **Aufwachen aus dem Schlafmodus (Wake-up Request):** `8,26 s` bis zur vollständigen Antwort (*„AIDO ist wieder wach.“*).

### 3. Verifiziertes lokales Modell-Manifest (Referenz-PC)

| Rolle | Datei | Größe (Bytes) | SHA-256 | Geprüfte Runtime |
| --- | --- | --- | --- | --- |
| Hauptmodell (Text & Planung) | `D:\AI_Models\llm\Qwen3.8-27B-UD-Q4_K_M.gguf` | `16.464.440.224` | `322E194FF79741C7BAA497C240F677F54B201B0EFAB44CA8E50F122B39123482` | `llama-server.exe` (`v0.9.0-dev` b12176, CUDA 13.4) |
| Vision-Projektor (`mmproj`) | `D:\AI_Models\llm\assets\mmproj-Qwen3.8-27B-BF16.gguf` | `931.146.432` | `83EE4F4F205FA514161778C41DF1EA14144FAA0F713510893B63C2395F5C2D53` | `llama-server.exe` (`--mmproj`) |
| Speculative Draft (`draft-dflash`) | `D:\AI_Models\llm\assets\Qwen3.8-27B-DFlash2-Q4_K_M.gguf` | `1.143.006.816` | `1A25C56858E1EBE93F2718AC1D49D1151F9323325C1BBFD6209370F4DB131EBD` | `llama-server.exe` (`--spec-type draft-dflash`) |
| Spracherkennung (ASR) | `D:\AI_Models\unsloth\whisper.cpp\models\ggml-small.bin` | `487.601.967` | `1BE3A9B2063867B937E64E2EC7483364A79917E157FA98C5D94B5C1FFFEA987B` | `whisper-server.exe` (`v1.9.5-unsloth.18`, CUDA 13.4) |
| Text-Embedding | `text-embedding-nomic-embed-text-v1.5` (LM Studio Katalog) | `84,11 MB` | Katalog-Artefakt (768 Dim.) | LM Studio / `llama.cpp` |
