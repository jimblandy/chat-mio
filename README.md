# chat-mio

This is a trivial chat program written in Rust, using the `mio` crate for asynchronous I/O.

To try it out, open two terminal windows. In one, type:

    $ cargo run -- --serve localhost:12345

In the other terminal window, type:

    $ cargo run -- localhost:12345

Then, whatever you type at one will be printed by the other.
Hit end-of-file (ctrl-D) to leave the chat.

You can accomplish exactly the same thing using no code at all with `nc`.
The point here was for me to get a feel for how mio works.
