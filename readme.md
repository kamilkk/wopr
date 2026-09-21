# WOPR - a z/OS-style training simulator in Rust

WOPR is a self-contained, deliberately-fake mainframe you run on your own machine to
learn how a z/OS estate fits together: the green-screen terminal, RACF security, TSO,
datasets, JES/JCL batch, an SMF audit trail, CICS, Db2, USS/OMVS, and a modern web tier —
all speaking in-character (`IKJ56500I`, `DFHAC2001`, `DSNE610I`, `RC=0000`). It's built in
Rust as a small workspace of focused crates, and it doubles as a hands-on tour of the
mainframe data model and protocols.

> `GREETINGS PROFESSOR FALKEN.  SHALL WE PLAY A GAME?`
> WOPR is inspired by *WarGames*.

---

## Quick start

**Prerequisites:** a recent Rust toolchain (`rustup`), and — only if you want a real green
screen — a 3270 emulator (`brew install x3270` on macOS gives `c3270`; `sudo apt install
c3270` on Debian/Ubuntu). Line-mode works with plain `nc`/`telnet`, no emulator needed.

```bash
git clone <your-repo> wopr-rs && cd wopr-rs
cargo build --workspace
cargo run -p wopr-server -- --secure       # or --training  (default is --secure)
```

The server logs its posture at start-up and binds three ports:

```
INFO wopr_server: scenario loaded mode=Secure
INFO wopr_server: security posture: hardened (no weaknesses active)
INFO wopr_server: selector listening on 2023
INFO wopr_server: DRDA listener on 50000
web tier on http://0.0.0.0:8080
```

**Log on (line mode, second terminal):**

```
$ nc 127.0.0.1 2023
WOPR  z/OS-STYLE TRAINING SYSTEM (SIMULATOR)
LOGON using L TSO, L CICS or L DB2.

Logon Type: L TSO
USERID: IBMUSER
PASSWORD: SYS1
READY
LISTDS 'IBMUSER.CUST.DATA'
IBMUSER.CUST.DATA
 RECFM=Fb  LRECL=80  BLKSIZE=800  DSORG=Ps
READY
SUBMIT 'IBMUSER.JCL(RUNIT)'
JOB00001 RUNIT    IBMUSER  RC=0000
READY
LOGOFF
```

**Web tier (REST + dashboard):**

```bash
curl -s http://127.0.0.1:8080/api/accounts/00001   # {"id":"00001","balance":1234.56}
curl -s http://127.0.0.1:8080/                       # dashboard HTML
```

**Default identities:** `IBMUSER / SYS1` (RACF `SPECIAL`) and `GUEST / GUEST` (no
attributes).

---

## What's inside

| Subsystem | Reach it with | Notes |
|-----------|---------------|-------|
| **VTAM selector** | `L TSO` · `L CICS` · `L DB2` | logon-by-applid |
| **RACF security** | `LISTUSER` | users, groups, attributes (`SPECIAL`/`OPERATIONS`/`AUDITOR`), logon |
| **TSO** | `READY` prompt | `HELP` `TIME` `PROFILE` `LISTUSER` `LISTDS` `LISTCAT` `SUBMIT` `ST` `SECEVENTS` `OMVS` `LOGOFF` |
| **Datasets** | `LISTDS` / `LISTCAT` | PS & PDS, `RECFM`/`LRECL`/`BLKSIZE`, catalog; a golden `FB`/80 record |
| **JES / JCL** | `SUBMIT` → `ST` | JCL parse + run (`IEFBR14`/`IEBGENER`/`IDCAMS`), spool, SDSF-style status, return codes |
| **SMF audit** | `SECEVENTS` | logon / job-submit events with detail, oldest-first |
| **CICS** | `L CICS` | `CESN`/`CESF` signon, `CEMT`/`CECI` gated behind signon |
| **Db2** | `L DB2` | `RUN SQL SELECT …` over `SYSIBM.*`, `SHOW`, `GRANT` + a **DRDA** `EXCSAT` responder on 50000 |
| **USS / OMVS** | `OMVS` from TSO | `ls` `cd` `cat` `id` `pwd`; UID/GID mapped from RACF |
| **Web tier** | `http://…:8080` | dashboard, CBSA-style REST, and a JWT `alg=none` security lab |
| **3270 data stream** | `tn3270` crate | CP037 EBCDIC, 12-bit addressing, `render_3270` / `parse_inbound` |
| **Scenarios** | `--secure` / `--training` | the same box, hardened or deliberately weak |

### The security angle: `--secure` vs `--training`

The `--training` scenario switches on deliberately weak configurations (as *data* inside
the sandbox — never attack tooling). The invariant that weaknesses apply **only** in
training mode is the whole lesson, and it's visible end to end:

```bash
cargo run -p wopr-server -- --secure     # console: "posture: hardened (no weaknesses active)"
#   nc … LISTUSER GUEST   ->   ATTRIBUTES=NONE

cargo run -p wopr-server -- --training   # console: WARN "WEAKNESS ACTIVE: GUEST over-privileged (OPERATIONS)"
#   nc … LISTUSER GUEST   ->   ATTRIBUTES=OPERATIONS
```

---

## Architecture

Four crates, split so the simulated OS never touches I/O:

```
wopr-rs/
├── crates/
│   ├── tn3270/       protocol library — telnet/TN3270, CP037 EBCDIC, the 3270 data
│   │                 stream (render/parse), IND$FILE. Reusable; also the basis for a
│   │                 future TN3270 *client*.
│   ├── wopr-core/    the simulated OS, no I/O — sessions & the VTAM-style selector, RACF,
│   │                 TSO, datasets/catalog, JES/JCL/spool, SMF audit, CICS, Db2 + DRDA,
│   │                 OMVS, and the secure/training scenario model.
│   ├── wopr-server/  the async binary (tokio) — binds the terminal (2023) and DRDA
│   │                 (50000) listeners, drives sessions, spawns the web tier, and ships
│   │                 the `woprctl` control CLI.
│   └── wopr-web/     the web tier — dashboard + REST + the JWT lab on 8080.
├── data/             persisted datasets (raw bytes, no translation) — see --dump-data
└── scenarios/        secure/training baselines (optional; the server also has built-ins)
```

The design keeps `wopr-core` pure (it decides *what to say*) and `wopr-server` I/O-bound
(it decides *how to say it*). Every environment emits an abstract `Screen`, so the same
state machine drives both line-mode text and — once the 3270 renderer is wired — a real
green screen.

---

## Ports

| Port  | Service |
|-------|---------|
| 2023  | terminal — TSO / CICS / DB2 (line mode; `nc`, `telnet`, or `c3270`) |
| 50000 | DRDA — answers an `EXCSAT` probe with `SRVNAM=WOPR` |
| 8080  | web tier — dashboard + REST + JWT lab |

---

## Building & testing

```bash
cargo build --workspace
cargo test  --workspace          # unit tests across every crate + a scripted integration test
./target/debug/woprctl status    # control CLI

# materialise the golden dataset as raw EBCDIC bytes (the Part 1 codec's input):
cargo run -p wopr-server -- --dump-data
xxd data/IBMUSER.CUST.DATA        # 80 bytes, 0x40 padding, no 0x0A — no translation
```

The integration test scripts a full `logon → LISTUSER → SUBMIT → SECEVENTS` flow against
the real state machine; the `tn3270` tests cover the EBCDIC codec, 3270 addressing and its
inverse, and the inbound Read-Modified round-trip.

---

## Scope & ethics

This is a **training simulator**: a deliberately-fake mainframe you run locally to learn.
It is **not** a real z/OS system and **not** a tool for testing systems you don't own. The
"training mode" models weak configurations *as data inside the sandbox* so you can study
cause, risk, and evidence safely — it does not produce attack tooling aimed at real hosts.
Keep it a lab.

---

## Credits

- Inspired by *WarGames* (WOPR).
- Protocol references: RFC 854 (Telnet), RFC 1576 / RFC 2355 (TN3270 / TN3270E), and IBM's
  *3270 Data Stream Programmer's Reference* (GA23-0059).
