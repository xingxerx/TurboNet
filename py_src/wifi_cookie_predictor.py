#!/usr/bin/env python3
"""
Quantum-Hound AI Cookie Predictor

Heuristic-based password prediction engine for Wi-Fi penetration testing.
Analyzes SSID patterns and OUI (vendor) information to predict likely passwords.

Part of TurboNet Quantum-Hardened Security Toolkit.

Usage:
    python wifi_cookie_predictor.py '{"ssid": "ATT123", "bssid": "00:11:22:33:44:55"}'
"""

import json
import re
import sys


# OUI Database: First 3 bytes of MAC -> Vendor -> Common Passwords
OUI_DATABASE = {
    # ISP Routers (Common default patterns)
    "00:1E:58": {"vendor": "D-Link", "passwords": ["admin", "password", "1234"]},
    "00:14:BF": {"vendor": "Linksys", "passwords": ["admin", "linksys", "password"]},
    "00:18:F3": {"vendor": "ASUSTek", "passwords": ["admin", "asus", "password"]},
    "00:1A:2B": {"vendor": "Ayecom", "passwords": ["admin", "password"]},
    "00:24:01": {"vendor": "D-Link", "passwords": ["admin", "dlink", "password"]},
    "00:26:F2": {"vendor": "Netgear", "passwords": ["admin", "password", "1234"]},
    "08:86:3B": {"vendor": "Belkin", "passwords": ["admin", "belkin", "password"]},
    "20:AA:4B": {"vendor": "Linksys", "passwords": ["admin", "linksys"]},
    "44:94:FC": {"vendor": "Arris", "passwords": ["password", "admin", "1234567890"]},
    "58:90:43": {"vendor": "Sagemcom", "passwords": ["admin", "password"]},
    "84:94:8C": {"vendor": "Arris", "passwords": ["password", "1234567890"]},
    "C0:A0:0D": {"vendor": "Arris", "passwords": ["password", "admin"]},
    "C8:D7:19": {"vendor": "Cisco-Linksys", "passwords": ["admin", "cisco"]},
    "E8:FC:AF": {"vendor": "Netgear", "passwords": ["admin", "password"]},
    "F8:E4:FB": {"vendor": "Actiontec", "passwords": ["password", "admin"]},
}

# SSID Pattern Heuristics
ISP_PATTERNS = {
    r"(?i)^att": {"isp": "AT&T", "passwords": ["password", "att", "attadmin"]},
    r"(?i)^xfinity": {"isp": "Xfinity", "passwords": ["comcast", "xfinity", "password"]},
    r"(?i)^spectrum": {"isp": "Spectrum", "passwords": ["spectrum", "charter", "password"]},
    r"(?i)^verizon": {"isp": "Verizon", "passwords": ["verizon", "fios", "password"]},
    r"(?i)^netgear": {"isp": "Netgear", "passwords": ["admin", "password", "netgear"]},
    r"(?i)^linksys": {"isp": "Linksys", "passwords": ["admin", "linksys", "password"]},
    r"(?i)^dlink": {"isp": "D-Link", "passwords": ["admin", "dlink", "password"]},
    r"(?i)^asus": {"isp": "ASUS", "passwords": ["admin", "asus", "password"]},
    r"(?i)^tp-?link": {"isp": "TP-Link", "passwords": ["admin", "tplink", "password"]},
    r"(?i)^telstra": {"isp": "Telstra", "passwords": ["telstra", "admin", "password"]},
    r"(?i)^optus": {"isp": "Optus", "passwords": ["optus", "admin", "password"]},
    r"(?i)^virgin": {"isp": "Virgin", "passwords": ["virgin", "password", "admin"]},
    r"(?i)^sky": {"isp": "Sky", "passwords": ["sky", "admin", "password"]},
    r"(?i)^bt-?hub": {"isp": "BT", "passwords": ["admin", "bthub", "password"]},
}

# Universal common passwords
COMMON_PASSWORDS = [
    "password",
    "12345678",
    "123456789",
    "1234567890",
    "admin",
    "password1",
    "qwerty123",
    "welcome",
    "letmein",
]


def get_oui(bssid: str) -> str:
    """Extract OUI (first 3 octets) from BSSID."""
    clean = bssid.upper().replace("-", ":").replace(".", ":")
    parts = clean.split(":")
    if len(parts) >= 3:
        return ":".join(parts[:3])
    return ""


def predict_cookie(ssid: str, bssid: str) -> list[str]:
    """
    Predict likely passwords (cookies) for a Wi-Fi network.
    
    Uses multiple heuristics:
    1. SSID-as-password variations
    2. ISP pattern matching
    3. OUI vendor database
    4. Common default passwords
    
    Returns: Ranked list of password predictions (most likely first)
    """
    predictions = []
    seen = set()
    
    def add_unique(pwd: str):
        if pwd and pwd not in seen:
            predictions.append(pwd)
            seen.add(pwd)
    
    # Heuristic 1: SSID-based passwords
    clean_ssid = ssid.lower().replace(" ", "").replace("-", "").replace("_", "")
    add_unique(clean_ssid)                    # SSID as password
    add_unique(clean_ssid + "123")            # Common suffix
    add_unique(clean_ssid + "1234")
    add_unique(clean_ssid.capitalize())       # Capitalized variant
    
    # Extract numbers from SSID (often used in passwords)
    numbers = re.findall(r'\d+', ssid)
    for num in numbers:
        add_unique(num)
        add_unique(num * 2)  # Repeated numbers
    
    # Heuristic 2: ISP pattern matching
    for pattern, info in ISP_PATTERNS.items():
        if re.match(pattern, ssid):
            for pwd in info["passwords"]:
                add_unique(pwd)
            break
    
    # Heuristic 3: OUI vendor lookup
    oui = get_oui(bssid)
    if oui in OUI_DATABASE:
        vendor_info = OUI_DATABASE[oui]
        for pwd in vendor_info["passwords"]:
            add_unique(pwd)
    
    # Heuristic 4: Common defaults
    for pwd in COMMON_PASSWORDS:
        add_unique(pwd)
    
    return predictions


def main():
    """CLI interface for Rust integration."""
    if len(sys.argv) < 2:
        print(json.dumps({"error": "No input provided"}))
        sys.exit(1)
    
    try:
        input_data = json.loads(sys.argv[1])
        ssid = input_data.get("ssid", "")
        bssid = input_data.get("bssid", "00:00:00:00:00:00")
        
        cookies = predict_cookie(ssid, bssid)
        
        # Output for Rust to parse
        result = {
            "ssid": ssid,
            "bssid": bssid,
            "predictions": cookies,
            "count": len(cookies),
        }
        print(json.dumps(result))
        
    except json.JSONDecodeError as e:
        print(json.dumps({"error": f"Invalid JSON: {e}"}))
        sys.exit(1)
    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)


if __name__ == "__main__":
    main()
