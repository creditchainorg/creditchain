# CreditChain Release Names — Ancient Greek City-States

Releases are named after the *poleis* of ancient Greece, taken roughly in order of
antiquity. The first mainnet release is **Argos**.

## Why this order

"Oldest" is genuinely contested, and the honest answer depends on what you count:
a continuously inhabited settlement, a Bronze Age palace centre, or a *polis* in
the classical political sense. Knossos and Mycenae are older as **sites** but were
Minoan and Mycenaean palace centres rather than city-states. Argos is the usual
answer for the oldest continuously inhabited Greek city that also became a polis —
which is why it leads.

The list below is ordered by that reading. It is a naming scheme, not a
historical claim, and the notes say what each name actually refers to.

## The sequence

| # | Name | Pronounced | Why it earns the slot |
|---|---|---|---|
| 1 | **Argos** | AR-goss | Among the oldest continuously inhabited cities in Europe (~5000 BC), and a true polis. **First mainnet release.** |
| 2 | **Knossos** | k'NOSS-oss | Crete. The Minoan palace centre — Europe's oldest known city, though not a polis. |
| 3 | **Mycenae** | my-SEE-nee | The Bronze Age power that named an age. Cyclopean walls, Lion Gate. |
| 4 | **Tiryns** | TEER-ins | Mycenaean citadel, walls so massive later Greeks credited giants. |
| 5 | **Pylos** | PY-loss | Nestor's seat; its archive of Linear B tablets is why we can read Mycenaean Greek. |
| 6 | **Thebes** | THEEBS | Boeotia's great power; broke Spartan hegemony at Leuctra. |
| 7 | **Athens** | ATH-ens | Democracy, drama, philosophy. Needs no introduction. |
| 8 | **Sparta** | SPAR-ta | The military polis. A society organised entirely around one idea. |
| 9 | **Corinth** | KOR-inth | The commercial hinge between mainland and Peloponnese. |
| 10 | **Miletus** | my-LEE-tus | Ionia. Thales, Anaximander — where Greek natural philosophy began. |
| 11 | **Ephesus** | EF-e-sus | Temple of Artemis; one of the ancient wonders. |
| 12 | **Delphi** | DEL-fye | The oracle; the *omphalos*, considered the centre of the world. |
| 13 | **Olympia** | o-LIM-pee-a | Sanctuary of Zeus and origin of the games. |
| 14 | **Syracuse** | SIR-a-kyooz | Sicily's Greek superpower; Archimedes' city. |
| 15 | **Rhodes** | ROADS | Maritime power; the Colossus. |
| 16 | **Samos** | SAH-moss | Pythagoras; the Eupalinian aqueduct, tunnelled from both ends and meeting in the middle. |
| 17 | **Chios** | KEE-oss | One of several claimants to Homer's birth. |
| 18 | **Naxos** | NAX-oss | Largest of the Cyclades; marble that built temples. |
| 19 | **Aegina** | ee-JY-na | Struck some of the earliest Greek coinage — a fitting name for a monetary release. |
| 20 | **Megara** | MEG-a-ra | Prolific coloniser; founded Byzantium. |
| 21 | **Eretria** | e-REE-tree-a | Euboea; early and far-reaching colonist. |
| 22 | **Chalcis** | KAL-sis | Eretria's rival across the Lelantine Plain. |
| 23 | **Sicyon** | SISH-ee-on | Renowned for sculpture and painting. |
| 24 | **Epidaurus** | ep-i-DAW-rus | Sanctuary of Asclepius; the theatre with perfect acoustics. |
| 25 | **Elis** | EE-lis | Administered the Olympic Games. |

## Convention

**Testnet and mainnet share the name.** A release is validated on
`<name>-testnet` and, once it holds, promoted to `<name>` on mainnet. So the
current work is **Argos testnet → Argos mainnet**: the same genesis structure,
the same node topology, the same contracts, with different keys.

Sharing the name is deliberate. It means "we tested this" is checkable rather
than asserted — anyone can compare the two chains and see whether the thing
promoted is the thing that was tested.

Format: `argos` for the release, `argos-testnet` / `argos` for the networks,
`v1.0.0-argos` for tags.

## Current

| Release | Testnet | Mainnet |
|---|---|---|
| **Argos** | building | **target — blocked on the key ceremony** |
| Knossos | — | — |

Mainnet Argos cannot be built until the offline key ceremony produces its
addresses. No tooling here generates or reads a mainnet private key.
