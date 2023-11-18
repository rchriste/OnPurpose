# Purpose

The purpose of this page is to explain how to run the SurrealDB against the data file directly. This allows you to inspect the database file directly and try out new queries.

Launch SurrealDB against the database as so

```PowerShell
surreal start -A -user root -pass root file://c:/.on_purpose.db
```

Then connect to SurrealDB using this program - https://github.com/StarlaneStudios/Surrealist/releases
It is also available as a website at <https://surrealist.app>

When I did this the first time the firewall blocked access and I needed to go into the windows firewall and allow surreal on a public network.
