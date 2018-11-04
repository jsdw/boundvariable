# Boundvariable (ICFP 06 Programming Challenge):

**Spoiler alert: below are my notes on progress through the challenge: avoid reading if you don't want spoliers!**

The challenge begins with implementing an interpreter following the spec um-spec.txt.

Running the interpreter on the Codex provided (and entering the decryption key provided of `(\b.bb)(\v.vv)06FHPVboundvarHRAk`) gives the ability to dump some data, which when inspected is seen to contain another program that itself can be run on the interpreter.

This _inner_ program provides a login prompt on running it. It says that one can login as `guest`. At this point I find that my program should not strip newlines, as the actual input required to pass this phase is `'guest\n'`. I'm in!