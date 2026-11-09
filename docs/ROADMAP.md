# Entwicklungsroadmap

Die Phasen sind in Abhängigkeitsreihenfolge geplant. Es gibt keine belastbaren Termine oder abgeschlossenen Meilensteine.

| Phase | Ziel | Nachweis für Abschluss |
| --- | --- | --- |
| 0 | Referenz-PC, vorhandene Modelllaufzeiten und Modelle gezielt inventarisieren; historischen Docker-Plan einordnen. | Versionen, Formate, Startzustand und offene Lücken dokumentiert. |
| 1 | Windows Bridge und Permission Gateway als minimale Basis. | Erlaubte, abgelehnte und bestätigungspflichtige Aktionen sind reproduzierbar unterscheidbar. |
| 2 | App Discovery und Capability Registry. | Apps werden erkannt; eine konkrete Fähigkeit wird mit Rechten und Teststatus angezeigt. |
| 3 | Smart Installer und Datenmodi. | Unterbrechung, Wiederaufnahme und Modi auf geeigneten Windows-11-Testsystemen geprüft. |
| 4 | Hauptmodell, lokales Text-Embedding, Bild-Eingang und Memory-Grundlage anbinden; Spracheingabe als eigenen Modellpfad prüfen. | Text- und Bildbeispiele, Transkription und Memory-Abruf laufen lokal; Startzeit, VRAM und Fehlerfälle sind gemessen. |
| 5 | Animierter Charakter, verständliches Onboarding und Privacy/Memory Center. | Testpersonen ohne Entwicklerwissen können die Kernaufgabe und ihre Freigaben nachvollziehen. |
| 6 | Stimme, Bildverständnis, proaktive Vorschläge und geplante Aufgaben. | Jede Funktion erfüllt eigene Datenschutz-, Berechtigungs- und Zuverlässigkeitstests. |
| 7 | Breitere App-Abdeckung und geprüfte Adaptererzeugung. | Adapter laufen isoliert durch Tests und werden erst nach Freigabe aktiviert. |
| 8 | Verpackung, Updates, Deinstallation und Releasevorbereitung. | Signierung, Wiederherstellung, Ressourcenmessung und Lizenzprüfung dokumentiert. |

## Unmittelbare nächste Schritte

**Ausführung auf Nutzerwunsch angehalten.** Die folgenden Punkte beschreiben die Reihenfolge bei Wiederaufnahme, nicht abgeschlossene Tests.

1. Den offengelegten LM-Studio-Testschlüssel widerrufen. Der erfolgreiche Embedding-Funktionstest ist im [Modellinventar](MODEL_RUNTIME_INVENTORY.md) dokumentiert; Wiederholbarkeit, Qualität und Offline-Datenfluss sind noch zu prüfen.
2. Die konkrete Qwen-Q4-Datei samt vorhandenem Projektor mit einem künstlichen Testbild prüfen. Ohne erfolgreichen Bildtest bleibt ihre Vision-Fähigkeit offen.
3. Das inzwischen heruntergeladene und per SHA-1 geprüfte mehrsprachige Whisper-`small` mit dem lokal erzeugten deutschen WAV transkribieren, sobald die Ausführung wieder aufgenommen wird. Die vorhandene `whisper-server.exe` lud das Modell, fand jedoch keine GPU; Audioverarbeitung ist noch ungetestet. Deutsche Windows-Stimmen erst für die Voice-Oberfläche bewerten.
4. Eine kleine, messbare Nutzeraufgabe für den ersten Prototyp wählen und Aktionsschema, Freigabegrenzen sowie lokale Protokollierung spezifizieren.
5. Hauptmodell- und Runtime-Kandidaten mit denselben Aufgaben auf Qualität, Startzeit, VRAM, Leerlauf-Entladung und Offline-Verhalten vergleichen. Ollama erst wieder einbeziehen, wenn sein fehlender Modell-Blob geklärt ist.
6. Den punktuellen Ist-Check in [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md) um Ubuntu-Benutzer, Docker-Integration und GPU-Passthrough ergänzen, soweit Docker für die Entwicklung gebraucht wird.

Eine breite App-Unterstützung wird erst nach einer zuverlässig geprüften Einzelintegration zugesagt.
