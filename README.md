# Car Client
Simple car route calculation with client-server architecture and using `basic_pathfinding` library.

Project Diagram:

![Project Diagram](image.png)

Map is encoded as simple string. Walls are represented as '#', walkable ways are represented as '.' character.

![Map Encoding](image-1.png)

After server started up, output of client is:

![Client & Server Output](image-2.png)

This is client code for route calculating example. Server is on this repo https://github.com/codesole/rust-networking