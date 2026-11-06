import fitz
import os
import re

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))

terms = [
    "S29JL064", "MN2DS", "MN103", "CN55001", "IC55007", "IC5001", "IC5002",
    "FIRMWARE", "FLASH", "UPDATE", "SELF-CHECK", "U99", "HDD FORMAT",
    "SERVICE MODE", "RAM DISK", "EEPROM", "NVRAM"
]

pdf_files = [
    os.path.join(PROJECT_ROOT, "manuals_and_schematics", "panasonic_dmr-eh55p9_eh55pc9.pdf"),
    os.path.join(PROJECT_ROOT, "manuals_and_schematics", "dmreh55.pdf")
]

for pdf_path in pdf_files:
    if not os.path.exists(pdf_path):
        continue
    print(f"=== Searching {pdf_path} ===")
    doc = fitz.open(pdf_path)
    print(f"Total pages: {len(doc)}")
    
    found = {t: [] for t in terms}
    for page_num in range(len(doc)):
        text = doc[page_num].get_text()
        for t in terms:
            if re.search(r'\b' + re.escape(t) + r'\b', text, re.IGNORECASE):
                found[t].append(page_num + 1)
                
    for t, pages in found.items():
        if pages:
            print(f"  [{t}]: {len(pages)} occurrences on pages: {pages[:10]}{'...' if len(pages)>10 else ''}")
