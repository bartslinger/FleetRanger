# Equipment

**Equipment** lists your remote controllers, drones and batteries, each on its own tab. FleetRanger adds them by itself from the serial numbers in the flight logs, so there is nothing to enter.

## Remote controllers

The **Remote controllers** tab shows each controller's serial number and name, whether it is paired, its flights and air time, who flew it last, its last flight and its last sync.

### Pair a controller

A controller uploads flight logs once it is paired. Select **Pair a controller**, choose how long the code is **Valid for** and, with several groups, the **Group** the controller joins. Then select **Create code** and type the code into the app on the controller. [Getting started](01-getting-started.md#pair-the-controller) has the steps.

Your open codes are listed under **Open codes**. Select the button on a code's row to revoke it. Revoking a code doesn't unpair a controller it already paired.

### Remove a pairing

To stop a controller from uploading, for example one that was lost, open it and select **Remove pairing**. Its flights stay. To use it again, pair it with a new code. Pilots can also unpair a controller in the app's **Settings**.

## Drones

The **Drones** tab shows each drone's serial number and name, its flights and air time, who flew it last and its last flight.

## Batteries

The **Batteries** tab shows each battery's serial number and name, the drone it was last in, its charge cycles, its health and its last flight. Health is the battery's full charge capacity as a share of its design capacity.

A battery's page has a chart of its full charge capacity over time, and a table of its flights with the charge at takeoff and landing, the highest temperature and the cycles. FleetRanger shows what the logs say and gives no warnings: the judgement is yours.

## A device's page

Select a row to open a controller, drone or battery. Its page shows its figures, with a link to its flights in [Flights](03-flights.md). Its flights, air time and cycles count all its flights, also those of other groups.

On the page you can give it a **Name**, such as the number painted on it, and, with several groups, move it to another **Group**. Select **Save**.

Who flew it last shows a name only if you may see that flight.

## Who may do what

- **Viewer**, **Editor** and **Admin** see the drones and controllers of their groups. Everyone also sees the equipment of the flights they can see. Admins see all equipment.
- **Pilot**, **Editor** and **Admin** pair controllers. Admins see and revoke the open codes of others too.
- **Editor** and **Admin** name drones and controllers.
- Only an **Admin** names batteries, moves drones and controllers to another group, and removes a controller's pairing.
