#!/usr/bin/env python3
"""ccq — the CreditChain quantum wallet.

Manages the post-quantum guardian keys that protect a QuantumGuard, using only
the Python standard library so it can run on an air-gapped machine with no
package installs and no network access.

    ccq new      --label mykey            derive a guardian key from a passphrase
    ccq new      --label mykey --random   derive one from system entropy instead
    ccq address  --label mykey            show the on-chain commitment (safe to publish)
    ccq arm      --label mykey ...        print the exact `cast send` to arm on-chain
    ccq sign     --label mykey --digest…  sign a break-glass digest OFFLINE
    ccq verify   --sig … --digest …       verify a signature locally
    ccq networks                          list networks and their chain ids

WHY A SEPARATE WALLET
---------------------
A guardian key is not an everyday key. It signs exactly once, in the worst moment
of an account's life, and its whole value is that it was never exposed before then.
That is the opposite of a hot wallet's threat model, so it gets its own tool with
its own rules:

  * The secret is never written to disk in `--passphrase` mode. It is re-derived
    from the passphrase each run, so the only copy lives in the owner's head or
    on paper.
  * `sign` never opens a socket. Move the digest in by hand, move the signature
    out by hand, and the secret never touches a networked machine.
  * Nothing here ever asks for, reads, or stores a secp256k1 private key. This
    wallet cannot move funds, which means a compromised copy of it cannot either.

CRYPTOGRAPHY
------------
WOTS+ (RFC 8391) over keccak256, n=32, w=16 — the same construction the on-chain
verifier implements, imported from the reference module so the two can never drift.
Security rests only on keccak256's preimage resistance, which Shor's algorithm does
not affect and Grover's only halves (256 -> ~128 bits).

Key derivation uses PBKDF2-HMAC-SHA512 at 600,000 iterations, matching current OWASP
guidance, with a domain-separated salt so a passphrase reused elsewhere yields an
unrelated key.

ONE-TIME — SAY IT AGAIN
-----------------------
A WOTS+ key signs ONE message. `sign` refuses to run twice for the same label
unless forced, because a second signature under the same key is what actually
breaks the scheme.
"""
import argparse
import hashlib
import json
import os
import pathlib
import stat
import sys

# wotsplus.py is a byte-identical copy of agent-finance/reference/wotsplus.py in
# creditchainorg/contracts, kept beside this file so ccq runs from one directory on an
# air-gapped machine. `cmp` the two before trusting either.
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from wotsplus import keygen, compress, sign as wots_sign, verify as wots_verify, keccak  # noqa: E402

HOME = pathlib.Path(os.environ.get("CCQ_HOME", pathlib.Path.home() / ".creditchain" / "quantum"))
KDF_ITERS = 600_000
DOMAIN = b"CreditChain-QuantumGuard-WOTSPlus-v1"

# The networks a guardian can be armed on. Chain ids are pinned here so `arm`
# can emit `cast send --chain <id>`: cast then refuses to broadcast if the RPC
# it reaches disagrees, which turns "I armed against the wrong endpoint" from a
# silent, permanent mistake into a failed command. Kept as a static table on
# purpose -- `arm` performs no network I/O, because this tool has to run on an
# air-gapped machine.
NETWORKS = {
    "devnet":  {"chain_id": 2026042403, "rpc": "https://devnet.creditchain.org",
                "release": "Argos", "live": True},
    "testnet": {"chain_id": 2026042404, "rpc": "https://testnet.creditchain.org",
                "release": "Argos", "live": True},
    "mainnet": {"chain_id": 2026042405, "rpc": None,
                "release": "Argos", "live": False},
}
DEFAULT_NETWORK = "testnet"

C = {"b": "\033[1m", "d": "\033[2m", "g": "\033[32m", "y": "\033[33m", "r": "\033[31m", "x": "\033[0m"}
if not sys.stdout.isatty():
    C = dict.fromkeys(C, "")


def die(msg):
    print(f"{C['r']}error:{C['x']} {msg}", file=sys.stderr)
    sys.exit(1)


def meta_path(label):
    return HOME / f"{label}.json"


def derive_from_passphrase(passphrase: str, label: str):
    """Deterministic key material. Nothing secret is stored; only the label is."""
    salt = DOMAIN + b"|" + label.encode()
    dk = hashlib.pbkdf2_hmac("sha512", passphrase.encode("utf-8"), salt, KDF_ITERS, dklen=64)
    return dk[:32], dk[32:]  # (secret seed, public seed)


def load_meta(label):
    p = meta_path(label)
    if not p.exists():
        die(f"no guardian key labelled '{label}'. Run: ccq new --label {label}")
    return json.loads(p.read_text())


def get_secret(meta, label):
    """Recover the secret seed, from the stored file or from the passphrase."""
    if meta["mode"] == "random":
        sp = HOME / f"{label}.seed"
        if not sp.exists():
            die(f"seed file missing: {sp}")
        raw = bytes.fromhex(sp.read_text().strip())
        return raw, bytes.fromhex(meta["pubSeed"][2:])
    pw = os.environ.get("CCQ_PASSPHRASE")
    if not pw:
        import getpass
        pw = getpass.getpass("passphrase: ")
    secret, pub = derive_from_passphrase(pw, label)
    if "0x" + pub.hex() != meta["pubSeed"]:
        die("wrong passphrase — derived a different key than the one on record")
    return secret, pub


# ── commands ─────────────────────────────────────────────────────────────────

def cmd_new(a):
    HOME.mkdir(parents=True, exist_ok=True)
    if meta_path(a.label).exists() and not a.force:
        die(f"'{a.label}' already exists. Use --force only if you are certain it is unused.")

    if a.random:
        secret, pub = os.urandom(32), os.urandom(32)
        sp = HOME / f"{a.label}.seed"
        sp.write_text(secret.hex())
        os.chmod(sp, stat.S_IRUSR | stat.S_IWUSR)  # 0600
        mode = "random"
    else:
        import getpass
        pw = os.environ.get("CCQ_PASSPHRASE") or getpass.getpass("choose a passphrase: ")
        if not os.environ.get("CCQ_PASSPHRASE"):
            if pw != getpass.getpass("confirm passphrase: "):
                die("passphrases did not match")
        if len(pw) < 12:
            die("use at least 12 characters — this key is the last line of defence")
        secret, pub = derive_from_passphrase(pw, a.label)
        mode = "passphrase"

    sk, pk = keygen(secret, pub)
    pk_hash = compress(pub, pk)
    meta = {"label": a.label, "mode": mode, "scheme": "WOTS+/keccak256/n32/w16",
            "pubSeed": "0x" + pub.hex(), "pkHash": "0x" + pk_hash.hex(), "used": False}
    meta_path(a.label).write_text(json.dumps(meta, indent=2))
    os.chmod(meta_path(a.label), stat.S_IRUSR | stat.S_IWUSR)

    print(f"\n{C['g']}✓ guardian key '{a.label}' created{C['x']}")
    print(f"  scheme    WOTS+ over keccak256 (post-quantum, hash-based)")
    print(f"  pkHash    {meta['pkHash']}   {C['d']}← publish this{C['x']}")
    print(f"  pubSeed   {meta['pubSeed']}   {C['d']}← publish this{C['x']}")
    if mode == "passphrase":
        print(f"\n{C['y']}Your passphrase IS the key.{C['x']} Nothing secret was written to disk.")
        print("Lose it and this guardian can never be used. Write it down, store it offline.")
    else:
        print(f"\n{C['y']}Secret seed written to {HOME / (a.label + '.seed')} (0600).{C['x']}")
        print("Back it up offline and delete it from any networked machine.")
    print(f"\n{C['d']}This key signs exactly ONCE. Guard it accordingly.{C['x']}")


def cmd_address(a):
    m = load_meta(a.label)
    print(json.dumps({k: m[k] for k in ("label", "scheme", "pkHash", "pubSeed", "used")}, indent=2))


def cmd_arm(a):
    m = load_meta(a.label)
    net = NETWORKS[a.network]

    if not net["live"]:
        die(f"{a.network} is not running yet, so there is nothing to arm against.\n"
            f"       Its genesis has not been signed -- see docs/quantum/LAUNCH.md.\n"
            f"       Arm on testnet first and rehearse a break-glass there; the same\n"
            f"       guardian key works on mainnet once it exists.")

    # --rpc overrides the network's default endpoint (a local node, say) but does
    # NOT override the chain id: the whole point is that the id is what gets checked.
    rpc = a.rpc or net["rpc"]

    print(f"\n{C['b']}Arm this guardian on {a.network} ({net['release']} release):{C['x']}\n")
    print(f"cast send {a.guard} \\\n"
          f"  'armGuardian(bytes32,bytes32,address,uint256,uint64)' \\\n"
          f"  {m['pkHash']} \\\n"
          f"  {m['pubSeed']} \\\n"
          f"  {a.recovery} \\\n"
          f"  {a.outflow_limit} {a.window} \\\n"
          f"  --chain {net['chain_id']} \\\n"
          f"  --rpc-url {rpc} --private-key $CONTROLLER_KEY\n")
    print(f"{C['d']}--chain {net['chain_id']} makes cast refuse if that RPC is a different "
          f"network.{C['x']}")
    print(f"{C['d']}Only the commitment goes on-chain. The secret stays here.{C['x']}")
    print(f"{C['y']}The recovery address is permanent — it cannot be changed after arming.{C['x']}")


def cmd_networks(a):
    """List known networks. Offline: prints the static table, queries nothing."""
    for name, n in NETWORKS.items():
        state = f"{C['g']}live{C['x']}" if n["live"] else f"{C['y']}not launched{C['x']}"
        default = f"  {C['d']}(default){C['x']}" if name == DEFAULT_NETWORK else ""
        print(f"  {C['b']}{name:<8}{C['x']} chain {n['chain_id']}  {n['release']:<6} {state}{default}")
        print(f"           {C['d']}{n['rpc'] or 'no public endpoint yet'}{C['x']}")


def cmd_sign(a):
    """Offline signing. Deliberately performs no network I/O of any kind."""
    m = load_meta(a.label)
    if m.get("used") and not a.force:
        die("this key has already signed. A second WOTS+ signature leaks enough to\n"
            "       forge others — that is a break, not an inconvenience. Arm a fresh guard\n"
            "       instead. Override with --force only for testing on a throwaway key.")
    digest_b = _hexarg(a.digest, 32, "digest")

    secret, pub = get_secret(m, a.label)
    sk, _ = keygen(secret, pub)
    sig = wots_sign(digest_b, sk, pub)

    if not wots_verify(digest_b, sig, pub, bytes.fromhex(m["pkHash"][2:])):
        die("self-check failed — refusing to emit a signature that does not verify")

    m["used"] = True
    meta_path(a.label).write_text(json.dumps(m, indent=2))

    out = "0x" + sig.hex()
    if a.out:
        pathlib.Path(a.out).write_text(out)
        print(f"{C['g']}✓ signature written to {a.out}{C['x']} ({len(sig)} bytes)")
    else:
        print(out)
    print(f"\n{C['d']}Submit with:  cast send <guard> 'breakGlass(bytes)' <signature>{C['x']}", file=sys.stderr)
    print(f"{C['d']}Anyone may relay it — funds can only go to the recovery address you committed.{C['x']}", file=sys.stderr)


def _hexarg(value, want_bytes, name):
    """Parse a hex argument, failing with a readable message rather than a traceback.

    Every one of these is operator-supplied at a keyboard, often by copy-paste from
    a terminal, so a truncated paste is the expected failure mode and must produce
    an explanation rather than a stack trace.
    """
    raw = value[2:] if value.startswith("0x") else value
    try:
        b = bytes.fromhex(raw)
    except ValueError:
        die(f"--{name} is not valid hex")
    if len(b) != want_bytes:
        die(f"--{name} must be {want_bytes} bytes, got {len(b)} "
            f"(a truncated copy-paste is the usual cause)")
    return b


def cmd_verify(a):
    digest = _hexarg(a.digest, 32, "digest")
    sig = _hexarg(a.sig, 2144, "sig")
    pub = _hexarg(a.pub_seed, 32, "pub-seed")
    pkh = _hexarg(a.pk_hash, 32, "pk-hash")
    ok = wots_verify(digest, sig, pub, pkh)
    print(f"{C['g']}✓ valid{C['x']}" if ok else f"{C['r']}✗ INVALID{C['x']}")
    sys.exit(0 if ok else 1)


def main():
    ap = argparse.ArgumentParser(prog="ccq", description="CreditChain quantum wallet")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("new", help="create a guardian key")
    p.add_argument("--label", required=True)
    p.add_argument("--random", action="store_true", help="use system entropy and store a seed file")
    p.add_argument("--force", action="store_true")
    p.set_defaults(fn=cmd_new)

    p = sub.add_parser("address", help="show the publishable commitment")
    p.add_argument("--label", required=True)
    p.set_defaults(fn=cmd_address)

    p = sub.add_parser("arm", help="print the on-chain arming command")
    p.add_argument("--label", required=True)
    p.add_argument("--guard", required=True)
    p.add_argument("--recovery", required=True)
    p.add_argument("--outflow-limit", default="10000000000000000000")
    p.add_argument("--window", default="86400")
    p.add_argument("--network", choices=sorted(NETWORKS), default=DEFAULT_NETWORK,
                   help=f"network to arm on (default: {DEFAULT_NETWORK})")
    p.add_argument("--rpc", default=None,
                   help="override the endpoint; the chain id is still pinned")
    p.set_defaults(fn=cmd_arm)

    p = sub.add_parser("networks", help="list known networks and chain ids")
    p.set_defaults(fn=cmd_networks)

    p = sub.add_parser("sign", help="sign a break-glass digest (offline)")
    p.add_argument("--label", required=True)
    p.add_argument("--digest", required=True)
    p.add_argument("--out")
    p.add_argument("--force", action="store_true")
    p.set_defaults(fn=cmd_sign)

    p = sub.add_parser("verify", help="verify a signature locally")
    p.add_argument("--digest", required=True)
    p.add_argument("--sig", required=True)
    p.add_argument("--pub-seed", required=True)
    p.add_argument("--pk-hash", required=True)
    p.set_defaults(fn=cmd_verify)

    a = ap.parse_args()
    a.fn(a)


if __name__ == "__main__":
    main()
