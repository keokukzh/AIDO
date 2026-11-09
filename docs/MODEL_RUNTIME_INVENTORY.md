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
4. Kurze reproduzierbare Tests für Text, Embedding und später ASR ausführen. Startzeit, Tokenrate, VRAM, CPU/RAM, Idle-Entladung und Fehler dokumentieren.
5. Erst danach über zusätzliche Downloads oder eine andere primäre Runtime entscheiden. Endnutzer-Installer sollen benötigte Modelle je nach gewählten Funktionen erkennen, wiederverwenden oder nach Zustimmung installieren.

Es wurde keine neue Modell- oder Runtime-Installation vorgenommen.

## Ausführung des Prüfplans am 9. November 2026

Die Benutzerumgebung setzt `OLLAMA_MODELS` auf `D:\AI_Models\ollama`; die laufende PowerShell-Sitzung hatte diese Variable nicht geerbt. Ein explizit mit diesem Pfad gestarteter temporärer Ollama-Server zeigte zweimal dieselben Einträge (`kimi-k2.6:cloud`, `qwen3.8:latest`). `ollama show qwen3.8:latest` scheiterte jedoch an einem fehlenden GGUF-Blob. Die frühere Liste mit GameDev-Qwen und qwen2.5 ließ sich diesem Zustand nicht zuordnen. Ollama wird für Modellvergleiche vorläufig ausgeschlossen; der Testserver wurde gestoppt. Es wurde nichts repariert oder gelöscht.

Ein `whisper-server.exe` wurde unter der bestehenden Unsloth/whisper.cpp-Installation gefunden. In den gezielt geprüften Modellverzeichnissen fand sich kein zugehöriges Whisper-Sprachmodell. Ein Sprachtest fand daher nicht statt.

Ein separater, nur an localhost gebundener Embedding-Testserver wurde durch die automatische Freigabeprüfung vor dem Start blockiert. Es gab keinen zweiten Startversuch; die LM-Studio-Tokenpflicht blieb aktiv. Diese Sperre betraf nur den separaten Testserver; der spätere autorisierte Embedding-Test gelang. Bild-Eingang bleibt ungetestet.

Die anschließend geöffnete LM-Studio-Oberfläche (Version 0.4.28) bestätigte einen laufenden lokalen Server auf `127.0.0.1:1234`, aktivierte Authentifizierung und neun vorhandene API-Schlüssel. Die Schlüssel wurden weder geöffnet noch ausgelesen. Die Detailansicht der GameDev-Qwen3.8-v2-REA-Q4-Datei zeigte als Fähigkeiten Tool Use und Reasoning, aber kein Vision. Der separate Projektor wurde in der Bibliothek als eigenes CLIP-Artefakt aufgeführt; eine erfolgreiche Zuordnung oder Bildverarbeitung wurde nicht beobachtet. Für den späteren Embedding-API-Test stellte der Nutzer einen Zugang bereit. Die Servereinstellungen blieben unverändert.

## Autorisierter Embedding-Test

Nach Bereitstellung eines temporären lokalen API-Zugangs lieferte das vorhandene Nomic-Embedding-Modell über LM Studios geschützten Endpunkt drei Vektoren mit je 768 Dimensionen. Für zwei thematisch verwandte deutsche Projekttexte betrug die Kosinusähnlichkeit 0,8036; für einen themenfremden Gartentext 0,5959. Damit ist eine einzelne Funktionsprobe samt Plausibilitätsvergleich bestanden. Reproduzierbarkeit, vollständige Prüfung der Vektorwerte, Suchqualität auf realen AIDO-Daten, Kaltstart und Offline-Datenfluss sind noch nicht belegt. Das Modell wurde anschließend entladen; der Token wurde nicht in Dateien oder das Repository übernommen. Weil der Nutzer den Token im Chat offengelegt hat, muss er in LM Studio widerrufen werden.
