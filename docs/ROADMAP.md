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

1. Den punktuellen Ist-Check in [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md) um Ubuntu-Benutzer, Docker-Integration und GPU-Passthrough ergänzen; Änderungen erst nach gesicherter Ausgangslage durchführen.
2. Das [Modellinventar](MODEL_RUNTIME_INVENTORY.md) um Ollamas abweichende Modellansichten ergänzen; die lokale Qwen-Ableitung mit passendem Projektor ausdrücklich mit Bild-Eingang testen.
3. Einen minimalen Modell-Spike definieren: Hauptmodell, Text-Embedding und Spracheingabe getrennt testen; Sprachausgabe erst für die Voice-Oberfläche auswählen.
4. Eine kleine, messbare Nutzeraufgabe für den ersten Prototyp wählen.
5. Aktionsschema, Freigabegrenzen und lokale Protokollierung spezifizieren.
6. Runtime-Kandidaten mit denselben Aufgaben auf Qualität, Startzeit, VRAM, Leerlauf-Entladung und Offline-Verhalten vergleichen.

Eine breite App-Unterstützung wird erst nach einer zuverlässig geprüften Einzelintegration zugesagt.
