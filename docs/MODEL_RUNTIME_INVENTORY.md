# Lokale Modelllaufzeiten: Inventar und nächster Prüfblock

**Stand: 9. November 2026.** Punktuelle Tests am Referenz-PC; kein vollständiger Benchmark und keine fertige AIDO-Integration.

## Vorhandene Laufzeiten

| Befund | Beobachtung |
| --- | --- |
| LM Studio | Installiert; lokaler Server meldete Port 1234 und verlangte für API-Anfragen ein Bearer-Token. |
| llama.cpp | In LM Studio sind mehrere llama.cpp-Engines installiert; ausgewählt war `llama.cpp-win-x86_64-nvidia-cuda12-avx2@2.71.0`. Ein separates `llama-server`/`llama-cli` wurde in den geprüften Windows- und Ubuntu-PATHs nicht gefunden. |
| Ollama | Windows-Installation vorhanden. Eine frühere Abfrage listete zwei Modelle; nach Start eines temporären Servers war dessen Liste leer. Die Ursache der unterschiedlichen Ansichten ist noch offen. Der temporäre Server wurde wieder gestoppt. |
| GPU | GeForce RTX 5090, 32607 MiB laut `nvidia-smi` zum Prüfzeitpunkt. |

Damit ist eine weitere llama.cpp-Installation **nicht** der erste Schritt. Zunächst ist die vorhandene LM-Studio-Engine als Referenz zu verwenden und Ollamas Modellpfad/Serverzustand zu klären.

## Modelle im LM-Studio-Katalog

- Mehrere Qwen3.8-27B-GGUF-Ableitungen waren vorhanden, darunter `gamedev-qwen3.8-v2-rea@q4_k_m` (16,81 GB) und eine Q6_K-Variante (22,43 GB).
- Ein separater `GameDev-Qwen3.8-v2-REA-mmproj.gguf`-Projektor (931 MB) war vorhanden.
- Das Text-Embedding-Modell `text-embedding-nomic-embed-text-v1.5` (84,11 MB) war bereits vorhanden. Für den Referenz-PC ist derzeit kein zusätzlicher Embedding-Download nötig.
- `lms ls --json` meldete für die gelisteten Qwen-Modelldateien `vision: false`. Das [offizielle Qwen3.8-27B](https://huggingface.co/Qwen/Qwen3.8-27B) ist bildfähig; die hier installierten Ableitungen und ihre Projektorzuordnung müssen separat geprüft werden.

## Durchgeführte Kurztests

Das vorhandene Embedding-Modell wurde in LM Studio geladen und danach wieder entladen. Ein API-Aufruf auf `/v1/embeddings` wurde wegen fehlendem Bearer-Token abgewiesen. Daher sind Vektorausgabe, Dimension und Suchqualität **noch nicht** getestet. Die Serverauthentifizierung wurde nicht verändert und kein Token ausgelesen.

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

1. Ollamas Server- und Modellpfade ohne Veränderung bestehender Modelle klären.
2. Für LM Studio einen autorisierten lokalen API-Test vorbereiten; Tokens weder in Git noch in Logs speichern.
3. Die Bildfähigkeit einer konkreten Qwen-Datei mit zugehörigem Projektor und Testbild verifizieren.
4. Kurze reproduzierbare Tests für Text, Embedding und später ASR ausführen. Startzeit, Tokenrate, VRAM, CPU/RAM, Idle-Entladung und Fehler dokumentieren.
5. Erst danach über zusätzliche Downloads oder eine andere primäre Runtime entscheiden. Endnutzer-Installer sollen benötigte Modelle je nach gewählten Funktionen erkennen, wiederverwenden oder nach Zustimmung installieren.

Es wurde keine neue Modell- oder Runtime-Installation vorgenommen.
