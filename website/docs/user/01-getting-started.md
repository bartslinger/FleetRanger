# Getting started

You have just logged in to your new organisation. These steps get the first flight in: put FleetRanger's app on a DJI remote controller, pair it, fly, and see the flight in a session.

Until your organisation has its first flight, FleetRanger opens on **Equipment**, which shows these steps under **Start by pairing a controller**.

## Before you start

The controller app shows a list of pilots to choose from. It holds everyone with the **Pilot** role. If you fly yourself, give yourself that role first: open **Users**, select your name, select **Edit user**, tick **Pilot** and **Save**. See [Users](04-users.md) to add other pilots.

## Install the app on the controller

1. On the controller, open the browser and go to your FleetRanger address followed by `/download`, such as `fleet.example.org/download`. The login page links there too, under **Download the app**.
2. Select **Download app** and install it. Android may ask you to allow installing apps from the browser first.
3. Open FleetRanger on the controller. It knows your FleetRanger address already. If it asks for a **Server address**, enter the one the download page shows.

## Pair the controller

Pairing connects the controller to your organisation. You create a code in the web app and type it on the controller.

1. In the web app, open **Equipment** and select **Pair a controller**.
2. Choose how long the code is **Valid for** (24 hours unless you change it). If your organisation has several groups, also choose the **Group** the controller joins.
3. Select **Create code**. The code has 8 letters and digits, such as `KDW4-9TXR`.
4. On the controller, type the code under **Pairing code** and select **Pair**.
5. The app says **Paired** and which group the controller is in. Select **Continue**. If the group is wrong, select **Unpair** and pair again with a new code.

A code pairs one controller, then stops working. Unused codes are listed under **Open codes** on the same page, where you can revoke them.

## Allow access to the flight logs

The app then asks for access to the flight logs that DJI Pilot 2 keeps on the controller. Select **Continue**, and choose **Allow** when Android asks. FleetRanger only reads the flight logs.

Next, choose **Which logs to upload**:

- **All logs**: every flight already on the controller.
- **From a date**: the logs from the chosen day on. Today is chosen to begin with.

Each choice shows how many logs it covers. You can change it later in the app's **Settings**.

## Choose who's flying

The app's main screen asks **Who's flying?**

- The first time, select **Other pilot** and choose the pilot from the list. After that, the last pilot is shown on the big tile, so the same pilot again is one tap.
- If the pilot isn't in the list, choose **Not in the list**. The flight then has no pilot, and an editor or admin sets it later.
- If your organisation has several groups, the app next asks where to **Log flights under**. It offers the groups the pilot flies for.

The tile that says **Open DJI Pilot 2** starts DJI Pilot 2. The choice counts until someone changes it.

> **Note:** If the app shows **FleetRanger doesn't open at startup**, follow its steps so the app opens when the controller switches on.

## Fly

Fly with DJI Pilot 2 as usual. After the flight, the app uploads the flight log by itself. By default it uploads over Wi-Fi networks that aren't metered; the app's **Settings** can allow other networks. The main screen shows how the upload is going, after **Sync:**.

## See the flight in a session

Once a log has been uploaded, FleetRanger reads it, which takes a moment. Admins see this as **Log processing** at the top of the page.

The flight then shows under **Sessions**. Flights by the same pilot with short breaks in between form one session. See [Sessions](02-sessions.md).

## Invite other people

Add the people who log in or fly under [Users](04-users.md). Pilots don't need a login: a name and the **Pilot** role put them in the controller's list.
