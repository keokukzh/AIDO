# Architekturentwurf

Dieses Dokument ist ein Planungsstand, keine Beschreibung vorhandener Software.

## Logische Komponenten

```text
Desktop-Charakter / Chat / optionale Sprache
                |
       AIDO Core / Task Engine
         |               |
 Permission Gateway   Memory Engine
         |               |
 Windows Bridge     Project Intelligence
         |
 App Intelligence / Capability Registry
         |
 API | CLI | geprüfter Adapter | UI Automation | Vision
```

Das Permission Gateway prüft jede Werkzeugaktion unabhängig vom Modell. Die Windows Bridge begrenzt Dateisystem-, Fenster- und Prozesszugriffe. Ein optionaler Admin Broker übernimmt nur gezielte erhöhte Aktionen nach Windows-Freigabe. Logische Komponenten müssen nicht je ein eigener Prozess sein.

## KI und Ressourcen

Ein lokales Hauptmodell ist die bevorzugte erste Konfiguration auf geeigneter Hardware. Qwen3.8-27B wurde im Planungschat als Kandidat für den Referenz-PC genannt; Hardwarepassung, Lizenz, Laufzeit und Qualität sind noch zu testen. Kleinere oder quantisierte Modelle sowie eine freiwillige Cloud-Option sollen andere Geräte abdecken. Das grosse Modell soll bei Leerlauf entladen werden können. Hermes und bestehende LM-Studio-/Ollama-Laufzeiten sind Prüfkandidaten, keine bestätigten Produktbestandteile.

## Modellrollen und Laufzeitwahl

Für den ersten Modell-Spike werden Hauptmodell, Text-Embedding-Modell für Memory-Suche und Spracherkennung getrennt betrachtet. Ein Embedding-Modell bildet Text für Ähnlichkeitssuche ab; gesprochenes Audio wird damit nicht transkribiert. Sprachausgabe benötigt bei Bedarf zusätzlich eine lokale TTS-Komponente. Das offizielle Qwen3.8-27B ist laut Modellkarte bildfähig; die konkrete lokale Ableitung und die gewählte Runtime müssen mit einem Bildtest geprüft werden. Bei GGUF können zusätzliche multimodale Dateien nötig sein. [Qwen-Modellkarte](https://huggingface.co/Qwen/Qwen3.8-27B), [llama.cpp-Server](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md).

Der Installer inventarisiert vorhandene Ollama-, LM-Studio- und llama.cpp-Laufzeiten, Modellformate, Versionen, API-Fähigkeiten und belegte Ressourcen. Er wählt einen primären Inferenzpfad nach Kompatibilitäts- und Leistungstests; zusätzliche Engines werden nur für klar abgegrenzte Funktionen genutzt. Für lokale Text-Embeddings ist Qwen3-Embedding-0.6B ein zu prüfender Kandidat, für Spracherkennung whisper.cpp mit passendem Sprachmodell. Dies sind keine festgelegten Produktabhängigkeiten. [Embedding-Modellkarte](https://huggingface.co/Qwen/Qwen3-Embedding-0.6B), [whisper.cpp](https://github.com/ggml-org/whisper.cpp).

## Memory

Für einen ersten Prototyp ist SQLite mit FTS5, strukturierten Projekt- und Systemdaten sowie lokalen Embeddings vorgesehen. Geplante Ebenen sind Grundregeln, Nutzer-, System-, Projekt-, Ereignis-, semantisches und Ablaufwissen. Ein Eintrag enthält nach Möglichkeit Quelle, Zeitpunkt und Gültigkeit. Abruf lädt nur relevanten Kontext; Konsolidierung entfernt Duplikate, ohne Originale stillschweigend zu löschen. Mem0 und ein Wissensgraph bleiben Evaluationskandidaten. Externe Defaults von Drittkomponenten müssen für Private Offline ausgeschlossen werden.

## App Intelligence

Die Capability Registry führt pro App geprüfte Aktionen und Rechte. Bevorzugt werden offizielle APIs und vorhandene CLIs, danach geprüfte Adapter; Windows UI Automation und visuelle Steuerung dienen als abgestufte Wege. Apps können, sofern zulässig, auch als spezialisierte Werkzeuge eingebunden werden. Automatisch generierte Adapter benötigen isolierte Tests, Versionsbindung und gesonderte Freigabe. CLI-Anything, API-Anything, WinApp CLI, Playwright und pywinauto sind Kandidaten zur Prüfung, keine integrierten Abhängigkeiten.

## Installer und Betrieb

Der Smart Installer prüft Hardware, Modelle und Apps, zeigt die Funktionen von Standard/Full Access und lässt Online- oder Offline-Einrichtung wählen. Er soll Downloadfortschritt und eine messwertgestützte Zeitprognose anzeigen, fehlgeschlagene Schritte wiederholen und unabhängige Teile weiter einrichten. Vorhandene Modelle und Programme werden wiederverwendet, wenn dies sicher und vom Nutzer gewählt ist. Deinstallation trennt AIDO-Komponenten von fremden Daten.

Als mögliche Speicherorte wurden `%LOCALAPPDATA%\AIDO` für Nutzerdaten und `%PROGRAMDATA%\AIDO` für geschützte Dienstdaten diskutiert. Dies ist noch kein festgelegtes Dateiformat. Docker/WSL2 dürfen für normale Nutzer keine Pflicht sein; eine Docker-Desktop-Integration ist ein mögliches App-Beispiel, keine bestätigte Funktion. Der nachgereichte Docker-Masterplan beschreibt eine frühere persönliche Entwicklungsumgebung. Seine Bausteine und der punktuelle Ist-Check sind in [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md) eingeordnet. Docker/WSL2 bleiben optionale Entwicklungswerkzeuge; die Windows-Bridge und Freigaben müssen nativ funktionieren.

## Offene technische Wahl

Tauri 2 mit Rust/TypeScript und Three.js mit glTF/VRM sind Entwurfskandidaten. Auswahl folgt einem Windows-Prototyp, Messungen zu Ressourcenverbrauch und einer Sicherheitsprüfung.
