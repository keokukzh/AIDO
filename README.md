# AIDO

AIDO ist ein geplantes Produkt für Windows 11: ein animierter Desktop-Begleiter, der Menschen ohne technische Vorkenntnisse bei der Bedienung ihres Computers unterstützt. Die KI soll vorzugsweise lokal laufen, sich nützliche Zusammenhänge merken, installierte Anwendungen erkennen und Aufgaben nach nachvollziehbarer Freigabe ausführen.

**Projektstand: Konzept und Architekturplanung.** Dieses Repository enthält derzeit keine AIDO-Anwendung, keinen Installer und keine nachgewiesenen Produkttests. Es ist eine Grundlage für die Entwicklung, keine Installationsanleitung.

## Grundprinzipien

- **Einfacher Einstieg:** Installation, App-Auswahl und Berechtigungen werden in verständlicher Sprache erklärt.
- **Lokal zuerst:** Für geeignete Hardware ist ein vollständig lokaler Betrieb das Ziel. Ein lokales Modell allein garantiert noch keinen Offline-Betrieb; alle Komponenten und Netzwerkwege müssen dafür geprüft werden.
- **Kontrollierbare Aktionen:** Zugriffsumfang und automatische Ausführung sind getrennte Einstellungen. Relevante Änderungen erfordern eine Freigabe.
- **Ehrliche Fähigkeiten:** AIDO soll pro Anwendung anzeigen, was erkannt, verbunden und tatsächlich getestet wurde.
- **Dauerhaftes Gedächtnis:** Wissen wird lokal, strukturiert und überprüfbar gespeichert; Nutzer können es einsehen, korrigieren und löschen.

## Dokumentation

| Dokument | Inhalt |
| --- | --- |
| [Vision](docs/VISION.md) | Produktidee, Zielgruppe und Grenzen |
| [Ziele](docs/GOALS.md) | Ziele und messbare Erfolgskriterien |
| [Anforderungen](docs/REQUIREMENTS.md) | Geplante Funktionen und Schutzanforderungen |
| [Architektur](docs/ARCHITECTURE.md) | Komponenten und technische Kandidaten |
| [Projektstatus](docs/PROJECT_STATUS.md) | Verifizierter Stand und offene Quellen |
| [Entwicklungsumgebung](docs/DEVELOPMENT_SETUP.md) | Früher Docker-Plan, aktueller Ist-Check und Übernahme für AIDO |
| [Roadmap](docs/ROADMAP.md) | Entwicklungsreihenfolge und Abnahme |
| [Entscheidungen](docs/DECISIONS.md) | Beschlüsse und ausdrücklich offene Fragen |

Die Dokumente beruhen auf den AIDO-Interviews und der Produktplanung vom 9. November 2026. Technische Bibliotheken und Modellnamen sind Kandidaten, keine fest eingebauten Abhängigkeiten. Der frühere Docker-Plan und der davor liegende Produktgedanke wurden nachgereicht und in der Entwicklungsumgebung eingeordnet. Aussagen über das Referenzsystem sind dort als punktuelle Beobachtungen markiert.

## Mitarbeit und Lizenz

Vor Implementierungsbeginn sind technische Machbarkeit, Sicherheitsgrenzen und Lizenzen externer Komponenten zu prüfen. Eine Produktlizenz ist noch nicht entschieden; dieses private Repository enthält daher bewusst keine LICENSE-Datei.
