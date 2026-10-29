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

![General tabblad](images/general.png)

- Toont het CPA-id, de status, de partijnamen, de party-id's, de typen van de party-id's, de startdatum en de einddatum.
- Typ een nieuwe waarde en verlaat het veld (druk op Tab of klik in een ander veld). De waarde wordt verwerkt zodra je het veld verlaat.
- Kies de status in de lijst **Status**: `proposed`, `agreed` of `signed`.
- Datums hebben de vorm `JJJJ-MM-DDTUU:MM:SSZ`, bijvoorbeeld `2027-03-01T12:00:00Z`. De `Z` betekent Zulu-tijd (UTC). In Nederland is dat 1 uur (winter) of 2 uur (zomer) vroeger dan de tijd op je klok.
- Onder elke datum staat hetzelfde moment twee keer: in Zulu-tijd en in de tijdzone die op je computer is ingesteld. Voorbeeld: `Zulu: 2026-10-09T10:44:32Z    Local: 2026-10-09 12:44:32 CEST (UTC+02:00)`. Is de datum ongeldig of ontbreekt de tijdzone, dan staat daar een melding.
- **Set to now** onder de startdatum vult de datum en tijd van nu in, in Zulu-tijd.
- **Set from certificates** onder de einddatum zet de einddatum op de verloopdatum van het certificaat dat als eerste verloopt. Alle certificaten van beide partijen tellen mee: leaf, intermediate en root. In het logvenster staat welk certificaat de datum heeft bepaald. Is dat certificaat al verlopen, dan verschijnt er een waarschuwing.

### Collaboration Role

![Collaboration Role tabblad](images/collaboration_role.png)

- Het tabblad heet **Collaboration Role (experimental)**: het toevoegen, wijzigen en verwijderen van Service actions en Collaboration Roles is nieuw. Controleer het resultaat op het tabblad **CPA Validation** voordat je de CPA opslaat.
- Toont per partij elke service (één rij per CollaborationRole, met servicetype, rol en proces) en daaronder de acties. **▶ Send** is een `CanSend`, **◀ Receive** een `CanReceive`. De kolommen tonen de actie, het channel, het package en de `OtherPartyActionBinding`.
- De kolom **Status** toont `OK` of het aantal problemen; een actie met een probleem is rood. Een probleem is een channel dat geen `DeliveryChannel` van de partij is, een package dat niet in de CPA staat, of een `OtherPartyActionBinding` die ontbreekt, niet naar de tegenovergestelde actie van de andere partij wijst, of waar niet naar terug wordt verwezen.
- Typ in **Filter** om alleen de services en acties te tonen waarvan de naam of het id de tekst bevat.
- Klik op een actie om die rechts te zien. Wijzig de actie, het package, het channel of de `BusinessTransactionCharacteristics` en klik op **Save Changes**. De lijst onder de knop noemt de problemen van de actie. Het id kan niet gewijzigd worden, zodat de andere partij ernaar blijft verwijzen.
- **Add Service action** (werkbalk, of rechtsklik op een service of actie) voegt een actie toe aan de geselecteerde service:
  - Kies **Send** of **Receive** en typ de actie. Het id wordt voorgesteld op basis van een actie met dezelfde richting in die service, zodat de naamgeving van de CPA behouden blijft; je kunt het aanpassen.
  - Package, channel en characteristics worden ingevuld vanuit de CPA; kies zo nodig andere waarden in de lijsten.
  - **Also add the opposite action** (standaard aan) voegt de bijbehorende actie toe bij de andere partij: een Send krijgt een Receive en andersom. Beide `OtherPartyActionBinding`-elementen verwijzen dan naar elkaar. De service van de andere partij wordt gevonden via de bestaande verwijzingen, anders via dezelfde servicenaam; kies zo nodig een andere in de lijst.
  - **Add** controleert of de id's geldig zijn en nog niet gebruikt worden. Bij een probleem blijft het venster open met een rode melding.

  ![Add Service action venster](images/service_action_dialog.png)

- **Add Collaboration Role** voegt een nieuwe service toe aan de partij van de geselecteerde rij, standaard ook bij de andere partij (met een eigen rolnaam). De velden worden ingevuld vanuit een bestaande Collaboration Role; **Generate UUID** vult de proces-uuid in.
- **Delete** (werkbalk, rechtsklik of de Delete-toets) verwijdert de geselecteerde actie of Collaboration Role na bevestiging. Bij een actie met een gespiegelde actie verwijdert **Yes** beide en **No** alleen de geselecteerde. Een partij houdt minstens één Collaboration Role.
- Wijzigingen worden in de geladen CPA gemaakt; gebruik **Save CPA** om ze naar een bestand te schrijven.

### Transport

![Transport tabblad](images/transport.png)

- Toont de transportelementen van beide partijen als boom.
- Klik op een regel met een waarde. Onder de boom staat dan `Editing:` gevolgd door de naam van het veld.
- Typ de nieuwe waarde in het veld onder de boom. Voor `certId` en `securityId` kies je een waarde uit de lijst; daarin staan de id's van die partij.
- Klik op **Save Changes** om de waarde te verwerken.

### Comment

![Comment tabblad](images/comment.png)

- Toont de opmerkingen in de CPA.
- **New**: typ een tekst in het veld onder de lijst en klik op **New**.
- **Update**: klik op een opmerking, pas de tekst aan en klik op **Update**.
- **Delete**: klik op een opmerking en klik op **Delete**.

### XML editor

![XML editor tabblad](images/xml_editor.png)

- Toont de volledige CPA als boom met elementen en attributen.
- **Een waarde wijzigen**: selecteer een regel, dubbelklik in de kolom **Value**, typ de nieuwe waarde en druk op Enter. Druk op Escape om te annuleren.
- **Toevoegen**: selecteer het bovenliggende element, typ de naam in het eerste veld en de waarde in het tweede veld, kies **Element** of **Attribute** en klik op **Add**.
- **Verwijderen**: selecteer een regel en klik op **Delete**.
- **Kopiëren**: selecteer een regel, klik er met de rechtermuisknop op en kies **Copy Value**.

### Certificates

![Certificates tabblad](images/certificates.png)

- Toont per partij elk certificaat-id (`CertId`) met de bijbehorende `KeyInfo`.
- **De gegevens van een certificaat bekijken**: open een `CertId` om per certificaat van de keten een regel **Certificate** te zien, met het type (leaf, intermediate of root), de common name en de verloopdatum. Open die regel voor subject, issuer, serienummer, **Valid from** en **Valid until**. Een verlopen certificaat is gemarkeerd met `(EXPIRED)`.
- **Zien welke certificaten worden gebruikt**: een certificaat waar geen enkel element van de CPA naar verwijst staat in grijs. Certificaten in de normale tekstkleur zijn in gebruik.
- **Een certificaat vervangen**: open een `CertId`, selecteer de regel **KeyInfo**, klik er met de rechtermuisknop op en kies **Upload Certificate**. Kies een PEM-bestand (`.cer`, `.crt` of `.pem`). Het bestand mag de hele keten bevatten; zet het leaf-certificaat bovenaan. Het certificaat moet een RSA-sleutel hebben.
- **Een certificaat downloaden**: selecteer de regel **KeyInfo**, klik er met de rechtermuisknop op en kies **Download Certificate**. Kies een bestandsnaam. Het bestand is een PEM-bestand met alle certificaten van dat `CertId`, het leaf-certificaat bovenaan. Alleen certificaten worden opgeslagen; een CPA bevat nooit een private key.
- **Kopiëren**: selecteer de regel **KeyInfo**, klik er met de rechtermuisknop op en kies **Copy KeyInfo**.

Na het vervangen van een certificaat kun je op het tabblad **General** met **Set from certificates** de einddatum bijwerken.

## 3. De CPA valideren

![CPA Validation tabblad](images/validation.png)

- Open het tabblad **CPA Validation** en klik op **Validate CPA**.
- `Geen fouten gevonden` betekent dat de CPA geldig is volgens het CPA-schema (`cpp-cpa-2_0.xsd`).
- Anders wordt de eerste fout getoond. Los die op en valideer opnieuw, totdat er geen fout meer overblijft.
- `Geen validatie uitgevoerd` betekent dat de geladen CPA nog niet is gevalideerd.

## 4. De CPA opslaan

- Klik linksonder in het venster op **Save CPA**.
- Kies een bestandsnaam en een locatie. Gebruik een nieuwe naam om het oorspronkelijke bestand te bewaren. Het bestand krijgt de extensie `.xml` als je er geen typt.
- In het logvenster staat `CPA saved to` gevolgd door de bestandsnaam als het bestand is geschreven. Mislukt het opslaan, dan verschijnt een foutmelding en blijft een bestaand bestand ongewijzigd.

## Logvenster en debugmeldingen

- Het logvenster boven de knoppen laat zien wat de applicatie heeft gedaan en welke fouten zijn opgetreden.
- Vink **Show debug messages** aan voor meer detail, bijvoorbeeld elk gewijzigd veld. Dit staat uit als de applicatie start.
- Met debugmeldingen aan toont **Set from certificates** ook alle certificaten met hun common name en einddatum, het certificaat dat het eerst verloopt bovenaan.

## Help

- **Help > Handleiding (Nederlands)** opent deze handleiding.
- **Help > How to use (English)** opent deze handleiding in het Engels.
- **Help > Readme** opent de technische readme (installatie, tests, build).
- Elke keer dat de applicatie start, zoekt zij de nieuwste versie op GitHub op. Is er een nieuwere versie, dan toont een pop-up beide versies met de knoppen **Download** en **Skip**. **Download** opent de download in je browser; **Skip** sluit de pop-up, en die verschijnt opnieuw als je de applicatie de volgende keer start. Er verschijnt niets als je de nieuwste versie al hebt of als er geen internetverbinding is.
- **Help > Check for updates** doet dezelfde controle op het moment dat jij dat wilt en toont het resultaat in het logvenster. Meldt de controle dat `api.github.com` niet gevonden kon worden (`getaddrinfo failed`), dan is er geen internetverbinding of kon de proxy van je netwerk niet gebruikt worden; open dan de releasepagina uit de melding in je browser. Op Windows worden de proxy-instellingen van Windows automatisch gebruikt, dus achter een bedrijfsproxy werkt de controle normaal gesproken gewoon.
- Testversies (prereleases en de nightly build) worden nooit als update aangeboden. Gebruik je zo'n versie, dan krijg je de volgende gewone release aangeboden.
- Alleen de versie wordt opgezocht; er wordt niets over je CPA verstuurd. De applicatie installeert zelf niets: vervang het oude programma door het gedownloade. De versie die je gebruikt staat in de titelbalk van het venster.

## Tips

- Rechtermuisknop op macOS: klik met twee vingers of houd Control ingedrukt terwijl je klikt.
- Valideer voordat je opslaat en bewaar een kopie van de oorspronkelijke CPA.
- Beide partijen moeten exact dezelfde CPA gebruiken. Stuur het opgeslagen bestand na een wijziging naar de andere partij.
