# Handleiding CPA Editor

Met de CPA Editor open je een ebXML CPA-bestand (Collaboration Protocol Agreement), pas je het aan en sla je het weer op als XML.

De knoppen en tabbladen in de applicatie zijn Engelstalig. In deze handleiding staan ze daarom met hun Engelse naam.

## 1. Een CPA openen

- Klik linksonder in het venster op **Load CPA**.
- Kies het CPA-bestand (`.xml`).
- In het logvenster boven de knoppen staat `CPA data loaded successfully.` als het bestand is geopend. Kan het bestand niet worden gelezen, dan verschijnt daar een foutmelding.

Als je een andere CPA laadt, wordt alles op het scherm vervangen. Wijzigingen die niet zijn opgeslagen gaan verloren.

## 2. De CPA aanpassen

Elk tabblad toont een deel van de CPA. Wijzigingen blijven in het geheugen staan totdat je de CPA opslaat (zie stap 4).

### General

- Toont het CPA-id, de status, de partijnamen, de party-id's, de typen van de party-id's, de startdatum en de einddatum.
- Typ een nieuwe waarde en verlaat het veld (druk op Tab of klik in een ander veld). De waarde wordt verwerkt zodra je het veld verlaat.
- Kies de status in de lijst **Status**: `proposed`, `agreed` of `signed`.
- Datums hebben de vorm `JJJJ-MM-DDTUU:MM:SSZ`, bijvoorbeeld `2027-03-01T12:00:00Z`. De `Z` betekent Zulu-tijd (UTC). In Nederland is dat 1 uur (winter) of 2 uur (zomer) vroeger dan de tijd op je klok.
- Onder elke datum staat hetzelfde moment twee keer: in Zulu-tijd en in de tijdzone die op je computer is ingesteld. Voorbeeld: `Zulu: 2026-10-09T10:44:32Z    Local: 2026-10-09 12:44:32 CEST (UTC+02:00)`. Is de datum ongeldig of ontbreekt de tijdzone, dan staat daar een melding.
- **Set to now** onder de startdatum vult de datum en tijd van nu in, in Zulu-tijd.
- **Set from certificates** onder de einddatum zet de einddatum op de verloopdatum van het certificaat dat als eerste verloopt. Alle certificaten van beide partijen tellen mee: leaf, intermediate en root. In het logvenster staat welk certificaat de datum heeft bepaald. Is dat certificaat al verlopen, dan verschijnt er een waarschuwing.

### Collaboration Role

- Toont de collaboration roles van beide partijen als boom. Dit tabblad is om te bekijken; gebruik het tabblad **XML editor** om deze waarden te wijzigen.

### Transport

- Toont de transportelementen van beide partijen als boom.
- Klik op een regel met een waarde. Onder de boom staat dan `Editing:` gevolgd door de naam van het veld.
- Typ de nieuwe waarde in het veld onder de boom. Voor `certId` en `securityId` kies je een waarde uit de lijst; daarin staan de id's van die partij.
- Klik op **Save Changes** om de waarde te verwerken.

### Comment

- Toont de opmerkingen in de CPA.
- **New**: typ een tekst in het veld onder de lijst en klik op **New**.
- **Update**: klik op een opmerking, pas de tekst aan en klik op **Update**.
- **Delete**: klik op een opmerking en klik op **Delete**.

### XML editor

- Toont de volledige CPA als boom met elementen en attributen.
- **Een waarde wijzigen**: selecteer een regel, dubbelklik in de kolom **Value**, typ de nieuwe waarde en druk op Enter. Druk op Escape om te annuleren.
- **Toevoegen**: selecteer het bovenliggende element, typ de naam in het eerste veld en de waarde in het tweede veld, kies **Element** of **Attribute** en klik op **Add**.
- **Verwijderen**: selecteer een regel en klik op **Delete**.
- **Kopiëren**: selecteer een regel, klik er met de rechtermuisknop op en kies **Copy Value**.

### Certificates

- Toont per partij elk certificaat-id (`CertId`) met de bijbehorende `KeyInfo`.
- **Een certificaat vervangen**: open een `CertId`, selecteer de regel **KeyInfo**, klik er met de rechtermuisknop op en kies **Upload Certificate**. Kies een PEM-bestand (`.cer`, `.crt` of `.pem`). Het bestand mag de hele keten bevatten; zet het leaf-certificaat bovenaan. Het certificaat moet een RSA-sleutel hebben.
- **Een certificaat downloaden**: selecteer de regel **KeyInfo**, klik er met de rechtermuisknop op en kies **Download Certificate**. Kies een bestandsnaam. Het bestand is een PEM-bestand met alle certificaten van dat `CertId`, het leaf-certificaat bovenaan. Alleen certificaten worden opgeslagen; een CPA bevat nooit een private key.
- **Kopiëren**: selecteer de regel **KeyInfo**, klik er met de rechtermuisknop op en kies **Copy KeyInfo**.

Na het vervangen van een certificaat kun je op het tabblad **General** met **Set from certificates** de einddatum bijwerken.

## 3. De CPA valideren

- Open het tabblad **CPA Validation** en klik op **Validate CPA**.
- `Geen fouten gevonden` betekent dat de CPA geldig is volgens het CPA-schema (`cpp-cpa-2_0.xsd`).
- Anders wordt de eerste fout getoond. Los die op en valideer opnieuw, totdat er geen fout meer overblijft.
- `Geen validatie uitgevoerd` betekent dat de geladen CPA nog niet is gevalideerd.

## 4. De CPA opslaan

- Klik linksonder in het venster op **Save CPA**.
- Kies een bestandsnaam en een locatie. Gebruik een nieuwe naam om het oorspronkelijke bestand te bewaren.

## Logvenster en debugmeldingen

- Het logvenster boven de knoppen laat zien wat de applicatie heeft gedaan en welke fouten zijn opgetreden.
- Vink **Show debug messages** aan voor meer detail, bijvoorbeeld elk gewijzigd veld. Dit staat uit als de applicatie start.
- Met debugmeldingen aan toont **Set from certificates** ook alle certificaten met hun common name en einddatum, het certificaat dat het eerst verloopt bovenaan.

## Help

- **Help > Handleiding (Nederlands)** opent deze handleiding.
- **Help > How to use (English)** opent deze handleiding in het Engels.
- **Help > Readme** opent de technische readme (installatie, tests, build).

## Tips

- Rechtermuisknop op macOS: klik met twee vingers of houd Control ingedrukt terwijl je klikt.
- Valideer voordat je opslaat en bewaar een kopie van de oorspronkelijke CPA.
- Beide partijen moeten exact dezelfde CPA gebruiken. Stuur het opgeslagen bestand na een wijziging naar de andere partij.
