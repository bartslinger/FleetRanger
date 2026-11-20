# Requirements

FleetRanger runs as three containers with Docker Compose: the backend, the web app and PostgreSQL. You need:

- **A Linux server** with Docker and its Compose plugin, x86_64 or arm64. 2 cores and 2 GB of memory are enough, plus disk space for the flight logs: usually a few MB per flight.
- **A domain name and HTTPS**: a reverse proxy in front of the containers. See [Install](02-install.md#https).
- **A login provider** that speaks OpenID Connect, such as Google. FleetRanger stores no passwords.
- **A DJI app key**, to decrypt flight logs. See below.
- **A Mapbox token**, for the maps. Without one, pages show no map.

Controllers reach the server over the internet, or over your own network if they only sync there.

## DJI app key

Flight logs from DJI Pilot 2 are encrypted. FleetRanger asks DJI for each log's keys, which takes an app key of your own:

1. Log in at [DJI Developer](https://developer.dji.com/user), creating an account if you have none.
2. Create an app with App Type **Open API**, and activate it through the link DJI emails you.
3. Copy the app's key, shown as the SDK key on your developer page, into `DJI_APP_KEY` (see [Configuration](03-configuration.md)).

The request to DJI holds the log's encrypted keys only, no positions or other flight data.

## Login

Register FleetRanger as a web application with your provider. The redirect URI is your address followed by `/auth/callback`, such as `https://fleet.example.org/auth/callback`. For Google, create an OAuth client of type "Web application" in the Google Cloud console; its issuer is `https://accounts.google.com`.

## Mapbox

Create a public token (`pk.…`) at [mapbox.com](https://account.mapbox.com) and restrict it to your address.
