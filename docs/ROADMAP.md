# Entwicklungsroadmap

Die Phasen sind in Abhängigkeitsreihenfolge geplant. Es gibt keine belastbaren Termine oder abgeschlossenen Meilensteine.

| Phase | Ziel | Nachweis für Abschluss |
| --- | --- | --- |
| 0 | Referenz-PC, vorhandene Modelle und Tools gezielt inventarisieren; historischen Docker-Plan einordnen; Anforderungen und Risiken präzisieren. | Inventar mit Quellen, offenen Lücken und freigegebenem Prototypumfang. |
| 1 | Windows Bridge und Permission Gateway als minimale Basis. | Erlaubte, abgelehnte und bestätigungspflichtige Aktionen sind reproduzierbar unterscheidbar. |
| 2 | App Discovery und Capability Registry. | Apps werden erkannt; eine konkrete Fähigkeit wird mit Rechten und Teststatus angezeigt. |
| 3 | Smart Installer und Datenmodi. | Unterbrechung, Wiederaufnahme und Modi auf geeigneten Windows-11-Testsystemen geprüft. |
| 4 | Lokale Modellanbindung, Textdialog und Memory-Grundlage. | Relevante Erinnerungen werden lokal gespeichert, gefunden, korrigiert und gelöscht. |
| 5 | Animierter Charakter, verständliches Onboarding und Privacy/Memory Center. | Testpersonen ohne Entwicklerwissen können die Kernaufgabe und ihre Freigaben nachvollziehen. |
| 6 | Stimme, Bildverständnis, proaktive Vorschläge und geplante Aufgaben. | Jede Funktion erfüllt eigene Datenschutz-, Berechtigungs- und Zuverlässigkeitstests. |
| 7 | Breitere App-Abdeckung und geprüfte Adaptererzeugung. | Adapter laufen isoliert durch Tests und werden erst nach Freigabe aktiviert. |
| 8 | Verpackung, Updates, Deinstallation und Releasevorbereitung. | Signierung, Wiederherstellung, Ressourcenmessung und Lizenzprüfung dokumentiert. |

## Unmittelbare nächste Schritte

1. Den punktuellen Ist-Check in [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md) um Ubuntu-Benutzer, Docker-Integration und GPU-Passthrough ergänzen; Änderungen erst nach gesicherter Ausgangslage durchführen.
2. Auf dem Referenz-PC Windows-Edition, Modelllaufzeiten und ausgewählte Apps mit Zustimmung erfassen; persönliche Nutzungsdaten nur nach Freigabe auswerten.
3. Eine kleine, messbare Nutzeraufgabe für den ersten Prototyp wählen.
4. Aktionsschema, Freigabegrenzen und lokale Protokollierung spezifizieren.
5. Kandidaten für UI, Runtime und Adapter mit kurzen Spikes vergleichen.

Eine breite App-Unterstützung wird erst nach einer zuverlässig geprüften Einzelintegration zugesagt.
