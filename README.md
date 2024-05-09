Fulcrum Test
============

Usage:

* Start server with `cargo run --bin server -- <DIR_TO_SERVE>`
  - optionally include a host and port too
* Start client with `cargo run --bin client -- <URL> <REMOTE_FILE_PATH> <SAVE_PATH>`

Notes:

Not a great deal to say here. The implementation is about as simple as I could think of while ticking all the boxes. 
* The protocol is just [length, filepath] from client and [length, file] from the server.
* The server streams the file into memory and straight out into the socket so arbitrarily large files should work fine. 
* The server tries to be resiliant to a misbehaving client - it won't let it access files outside of the hosted directory.
* Upon error, the client will remove incomplete files
