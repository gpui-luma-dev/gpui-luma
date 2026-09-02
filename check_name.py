#!/usr/bin/env python3
"""Check availability on crates.io and across DNS/RDAP."""

import argparse
import concurrent.futures
import json
import socket
import sys
import urllib.error
import urllib.request

TLDS = ["org", "com", "dev", "io"]
USER_AGENT = "name-checker-cli/1.0 (contact: user@example.com)"

GREEN = "\033[32m"
RED = "\033[31m"
YELLOW = "\033[33m"
RESET = "\033[0m"


def check_crates_io(name: str) -> tuple[str, str]:
    """Check crates.io API. Handles hyphen/underscore equivalence."""
    # crates.io normalizes '_' to '-' for slug uniqueness
    canonical_name = name.lower().replace("_", "-")
    url = f"https://crates.io/api/v1/crates/{canonical_name}"
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})

    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            if resp.status == 200:
                data = json.loads(resp.read().decode())
                crate_id = data.get("crate", {}).get("id", canonical_name)
                return "crates.io", f"{RED}TAKEN{RESET} (https://crates.io/crates/{crate_id})"
    except urllib.error.HTTPError as e:
        if e.code == 404:
            return "crates.io", f"{GREEN}AVAILABLE{RESET}"
        return "crates.io", f"{YELLOW}HTTP {e.code}{RESET}"
    except Exception as e:
        return "crates.io", f"{YELLOW}Error: {e}{RESET}"

    return "crates.io", f"{YELLOW}UNKNOWN{RESET}"


def check_dns_and_rdap(domain: str) -> tuple[str, str]:
    """1. Fast DNS socket check (if it resolves, it's taken).

    2. If no DNS records, query authoritative RDAP to verify registration.
    """
    # Quick check: does it resolve to an IP or nameserver?
    try:
        socket.gethostbyname(domain)
        return domain, f"{RED}TAKEN (DNS Resolves){RESET}"
    except (socket.gaierror, UnicodeError):
        pass  # Doesn't resolve; could still be registered/parked without DNS

    # Authoritative check: RDAP (RFC 7482) via rdap.org
    url = f"https://rdap.org/domain/{domain}"
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(req, timeout=6) as resp:
            if resp.status == 200:
                return domain, f"{RED}TAKEN (Registered){RESET}"
    except urllib.error.HTTPError as e:
        if e.code == 404:
            return domain, f"{GREEN}AVAILABLE{RESET}"
        return domain, f"{YELLOW}INCONCLUSIVE (HTTP {e.code}){RESET}"
    except Exception:
        # Fallback if RDAP times out but DNS didn't resolve
        return domain, f"{YELLOW}NO DNS (Verify with registrar){RESET}"

    return domain, f"{YELLOW}UNKNOWN{RESET}"


def inspect_name(candidate: str):
    name = candidate.strip().lower()
    print(f"\n=== Checking: \033[1m{name}\033[0m ===")

    domains_to_check = [f"{name}.{tld}" for tld in TLDS]

    with concurrent.futures.ThreadPoolExecutor(max_workers=5) as executor:
        crate_future = executor.submit(check_crates_io, name)
        domain_futures = [executor.submit(check_dns_and_rdap, d) for d in domains_to_check]

        # 1. Output Crates.io
        target, status = crate_future.result()
        print(f"  {target:<14} -> {status}")

        # 2. Output Domains
        for future in domain_futures:
            domain, status = future.result()
            print(f"  {domain:<14} -> {status}")


def main():
    parser = argparse.ArgumentParser(
        description="Verify crate name and domain availability (.org, .com, .dev, .io)."
    )
    parser.add_argument("names", nargs="+", help="Candidate names to check (e.g. clyke zelta)")
    args = parser.parse_args()

    for name in args.names:
        inspect_name(name)


if __name__ == "__main__":
    main()
