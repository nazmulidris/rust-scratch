# tcp-api-server

```
┌──────────────────┐ ┌────────────────────────────────────┐ ┌─────────────────────────────┐
│                  │ │                                    │ │                             │
│  CLIENT          │ │  TCP Protocol                      │ │  Server                     │
│  - API           │ │  - bincode to handle enum          │ │  - expose API over TCP      │
│  - CLI (tuify)   │ │  - length first prefix (bigendian) │ │  - use kv for persistence   │
│                  │ │                                    │ │                             │
└──────────────────┘ └────────────────────────────────────┘ └─────────────────────────────┘
```

<!-- Source diagram:
https://asciiflow.com/#/share/eJzFk8FqwkAQhl9lmJOCuRQkNDeRHoQigXrcS0wmunSdDZuNJIggPkEPHvowPo1P0qT0oGQhEAld%2FsMsMzvf%2FgxzQI52hAEXSk1QRRUZDPAgsBQYvE79icCqjl78JrJU2voi8Ha59hY887iXvoZFCsGNrdb589pOuErPnaV3pPn74m25cpBW8xBCo62OteogfZDZk%2BkieTALFy5PHqwlxzohsBq2ESeKgLjYOUgeUJnpnH5b6QbafLNNql3ByBYyrcaPJEW8sVtIpcktZIZSWcJoLTfEiYx4fE8qasznHlJtICOTy9wSx%2FRIctgdZE63y6mX%2FmFDvgfeEDzi8QduiDC9)
-->

