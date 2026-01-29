A self assessment tool for learning Latin. In theory this could be expanded to
support other language, but in earnest I don't think I will ever expand the
scope of this anytime soon.

You can manage the installation via the `init`/`nuke` commands, but to be honest
doing so will be quite a chore considering all the forms, words, etc. to be
entered before all of this starts to be useful. That's why I simply move around
a backup database file (since this is all based on SQLite3). You can find a
reasonibly-sized one at `testdata/test.sqlite3`. Just move it to your
installation path and that should do it.

You can then manage `words`, `tags` and `exercises` via their own
commands. Finally, you run practices via the `practice` command (which is also
the default command), which can be tweaked via multiple flags in order to tailor
a practice session to your will.

In a practice session this tool will make a best effort to ask for stuff that
was last asked, or stuff in which you recently failed at. A word or exercise
won't be marked as "solved for now" until it has not been marked successfully
multiple times. Hence, there's a feel for iterating on words/exercises until you
pass them. If you feel like you want to double down on some specific word that
is not being asked, you can also call the `words poke` command, which will force
the next session to ask for it.

## License

This repository holds two licenses, as you can also note on the `Cargo.toml`
file. As it's written there:

- The source code on the `crates/` directory is licensed under the GNU GPLv3 (or
  any later version).
- The source code on the `lib/` directory is licensed under the GNU LGPLv3 (or
  any later version).

In practice, for the libraries under `lib/` this means that if you plan to
compile your binary statically, you still need to abide by the LGPLv3+ license.
This means at least providing the object files necessary to allow someone to
recompile your program using a modified version of these libraries. See the
LGPLv3 license for more details.
