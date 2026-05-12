#!/usr/bin/env python3
"""
Submit Jakob's Tana quest nodes to the live QuestHub API.
Run once to seed the garden with real aspirations.
"""

import json, time, urllib.request, urllib.error

API = "https://questhub.eco/api/quests/json"

QUESTS = [
    # ── Technology & Infrastructure ──────────────────────────────────────────
    {
        "quest": "Open-Source Car Designs",
        "category": "tech",
        "description": "Make the mechanical and software blueprints of road vehicles publicly available under open licenses - so anyone can build, repair, or improve them. Combines hardware openness with the right to repair. Reduces dependency on proprietary supply chains and enables community-driven safety improvements.",
    },
    {
        "quest": "Buy and Open-Source the Sono Car",
        "category": "tech",
        "description": "The Sono Sion - a solar-integrated car cancelled in 2023 - had genuine promise and thousands of pre-orders. The quest: acquire the IP, release the full engineering specs as open-source, and let communities build it. A test case for rescuing stranded innovation through collective ownership.",
    },
    {
        "quest": "LibrePhone Project",
        "category": "tech",
        "description": "A fully open-source smartphone from hardware to OS - no proprietary blobs, no lock-in. Community-auditable baseband, documented schematics, and a governance model that keeps it sovereign. Builds on Replicant, PinePhone, and Fairphone learnings but goes further on openness.",
    },
    {
        "quest": "Modular Phone with Upgradeable Components",
        "category": "tech",
        "description": "A smartphone designed like LEGO - swap the camera module, upgrade the battery, replace only the screen. Real modularity, not the Project Ara marketing version. Addresses e-waste at the source and decouples device longevity from manufacturer update cycles.",
    },
    {
        "quest": "Extending the FUTO Keyboard",
        "category": "tech",
        "description": "FUTO Keyboard is a rare open-source mobile keyboard with voice input. The quest: extend it with multilingual voice correction, clipboard history, and optional on-device LLM suggestions - keeping all processing local. Contributes to a privacy-first input layer for mobile devices.",
    },
    {
        "quest": "A Green, Universal Laptop Design",
        "category": "tech",
        "description": "An open-hardware laptop with: repairability from day one, standardised battery and port form factors, conflict-free material sourcing, and a community governance board. Not just a Fairphone for laptops - a reference design others can manufacture and improve.",
    },
    {
        "quest": "Upgradeable Chips on a Sleek Exterior",
        "category": "tech",
        "description": "Consumer devices that separate aesthetic shell from compute module - so you keep the body for 10 years and only replace the silicon when needed. Requires standardised compute modules and a design language that makes upgrade panels normal. Radical longevity by design.",
    },
    {
        "quest": "Modular Compute Module Standard",
        "category": "tech",
        "description": "A shared open standard for swappable compute modules - so laptops, desktops, and embedded devices can share the same CPU/RAM card. Like SODIMM but for entire compute units. Needs cross-manufacturer agreement and a governance body. Directly enables the upgradeable devices quest.",
    },
    {
        "quest": "Firefox-Based BrowserOS",
        "category": "tech",
        "description": "A lightweight OS where the browser is the shell - built on Firefox/Gecko to escape Chromium monoculture. Targeted at low-spec devices and privacy-conscious users. Combines Flatpak-style app isolation with a web-first interface and no Google bundling.",
    },
    {
        "quest": "OpenWindows - A Community Windows Fork",
        "category": "tech",
        "description": "A community-maintained, de-telemetrised fork of Windows that removes forced updates, ads, and data collection while remaining compatible with existing software. The goal is not another Linux distro - it is Windows-as-it-was-promised, governed by users.",
    },
    {
        "quest": "OpenMac - Auditable macOS Builds",
        "category": "tech",
        "description": "Community-institution-approved macOS builds with a public audit ledger - so security researchers and enterprises can verify what shipped. Closes the gap between Apple's closed update process and the need for reproducible, inspectable OS releases for regulated environments.",
    },
    {
        "quest": "Copyright-Respecting LLM",
        "category": "tech",
        "description": "A large language model trained exclusively on consented or permissively licensed data, with full training provenance published. Every token traced, every dataset audited. Demonstrates that capable AI does not require harvesting the world's text without permission.",
    },
    {
        "quest": "End-to-End Encrypted Calendar",
        "category": "tech",
        "description": "A calendar where the server never sees your events - encryption and decryption happen on-device, keys stay with the user. Compatible with existing CalDAV clients. Fills the gap between Proton Calendar (good but siloed) and fully open, self-hostable e2ee scheduling.",
    },
    {
        "quest": "Proton Calendar and CalDAV Interoperability",
        "category": "tech",
        "description": "Today Proton Calendar is walled off from standard CalDAV clients. The quest: a full bidirectional CalDAV bridge so encrypted Proton events sync to Thunderbird, Apple Calendar, and any open client. Breaks the privacy-vs-openness false trade-off.",
    },
    {
        "quest": "Interoperable Calendar Standard",
        "category": "tech",
        "description": "A shared protocol layer so Google Calendar, Proton, Fastmail, and self-hosted instances can exchange events with full fidelity - recurrences, attachments, attendee responses. Extends CalDAV with a trust and identity layer. Ends the calendar fragmentation that forces lock-in.",
    },
    {
        "quest": "Interoperable Encrypted Filesystem Standard",
        "category": "tech",
        "description": "A shared open standard for encrypted filesystem containers - readable and writable on macOS, Linux, and Windows without third-party drivers. Replaces the VeraCrypt ecosystem with something that ships in all OS kernels. Extends VeraCrypt's model to universal read-write support.",
    },
    {
        "quest": "Extend VeraCrypt to All Operating Systems",
        "category": "tech",
        "description": "VeraCrypt is the gold standard for encrypted volumes but its cross-platform support is uneven and the UX is stuck in 2010. The quest: modernise the GUI, add iOS and Android support, integrate with system keychain, and contribute upstream maintenance capacity.",
    },
    {
        "quest": "Universal Read-Write External Drive Format",
        "category": "tech",
        "description": "A file system format for external hard drives that is fully read-write on macOS, Linux, and Windows out of the box - no NTFS-3G hacks, no exFAT patent worries, no ext4 third-party drivers. An obvious gap that has resisted standardisation for 20 years.",
    },
    {
        "quest": "Per-Person Timeline",
        "category": "tech",
        "description": "A private, locally-stored chronological log of everywhere you have been - like Google Timeline but self-hosted, open-source, and never leaving your device. Imports from GPS logs, photos, and check-ins. Useful for memory, insurance claims, and personal history.",
    },
    {
        "quest": "Lean Wireshark Default Mode",
        "category": "tech",
        "description": "Wireshark is powerful but overwhelming for the 80% use case of 'what is this process calling home?'. The quest: a curated default mode that shows only significant traffic with plain-language explanations of suspicious patterns - like a security dashboard, not a packet dump.",
    },
    {
        "quest": "Deduplication Engine for the Web Archive",
        "category": "tech",
        "description": "The Internet Archive contains vast amounts of duplicate content from mirror sites and re-hosts. A smart deduplication pass - with provenance tracking so originals are preserved - would meaningfully reduce storage cost and improve search quality across decades of saved web.",
    },

    # ── Tools & Products ─────────────────────────────────────────────────────
    {
        "quest": "Resilient Collaboration Without Google Sheets",
        "category": "tools",
        "description": "A genuinely offline-first, conflict-free collaborative spreadsheet that works without Google. Combines CRDTs for real-time sync, end-to-end encryption for shared workspaces, and an import path from Sheets. Not a worse copy - a structurally different approach to shared tabular data.",
    },
    {
        "quest": "Adapter Between Logseq and Obsidian",
        "category": "tools",
        "description": "A live sync bridge so a Logseq graph and an Obsidian vault stay in sync - translating block references, page links, and properties in both directions. Lets teams where different members prefer different tools collaborate on one knowledge base without losing structure.",
    },
    {
        "quest": "Transcription for Jitsi",
        "category": "tools",
        "description": "Real-time and post-meeting transcription built into Jitsi Meet - fully on-premise, no Zoom, no Google Meet. Speaker diarisation, automatic summary, and export to Markdown. Makes self-hosted video conferencing a complete alternative for organisations that need meeting records.",
    },
    {
        "quest": "Word Count and Writing Stats on Zed",
        "category": "tools",
        "description": "Zed editor has no word count or writing statistics. For writers using Zed for long-form work, this is the one missing piece. A plugin or built-in feature: live word count, reading time, daily writing goal tracker - without leaving the editor.",
    },
    {
        "quest": "Recreate Wispr Flow Open-Source",
        "category": "tools",
        "description": "Wispr Flow is a macOS voice-to-text tool that works in any text field. The quest: build an open-source equivalent - local Whisper model, system-wide dictation, learns your vocabulary, no cloud dependency. Handy (by cjpais) is close; the gap is feature parity and polish.",
    },
    {
        "quest": "Contribute to Handy for Wispr Flow Parity",
        "category": "tools",
        "description": "Handy is an open-source voice input tool that nearly matches Wispr Flow. The remaining gaps: global hotkey reliability on all macOS versions, custom vocabulary injection, and a clean preference pane. Contributing these upstream closes the last reason to use proprietary dictation.",
    },
    {
        "quest": "Distraction-Free Mode for Signal",
        "category": "tools",
        "description": "A mode in Signal that silences all notifications and hides unread counts for a chosen duration - deep work compatible messaging. Optionally shows only messages from a starred contacts list. The goal: use Signal for important communication without it becoming another distraction source.",
    },
    {
        "quest": "Framework for Context Switching",
        "category": "tools",
        "description": "A system - part software, part protocol - for switching between work contexts without losing state. Closes all the right apps, saves browser sessions, parks unfinished tasks, then restores the full environment when you return. The missing layer between OS and productivity workflow.",
    },
    {
        "quest": "Version History for Passwords",
        "category": "tools",
        "description": "Password managers discard old passwords on update. The quest: a password vault with full version history per entry - so you can recover access to old accounts that never got the change notification, and audit when each credential was created or modified.",
    },
    {
        "quest": "Application Helper Development",
        "category": "tools",
        "description": "An in-app assistant layer that knows the current application's full capability - shortcuts, hidden features, obscure settings - and surfaces contextual suggestions as you work. Not a general AI - a deep per-app expert that reduces the documentation gap for power users.",
    },

    # ── Society & Politics ───────────────────────────────────────────────────
    {
        "quest": "Trustworthy Food System",
        "category": "society",
        "description": "An end-to-end food provenance system where ingredients are traceable from field to fork - with independent lab audits, not just supply chain claims. Combines QR codes, open databases, and community verification to make 'locally grown' and 'no pesticides' statements checkable by anyone.",
    },
    {
        "quest": "Repair Culture Infrastructure",
        "category": "society",
        "description": "A city-scale system supporting repair over replacement - repair cafes, certified repair vouchers, spare parts libraries, and social recognition for people who fix things. Combines policy advocacy with platform tools to make repair the culturally normal choice.",
    },
    {
        "quest": "Trust-Based Marketplace (Willhaben Alternative)",
        "category": "society",
        "description": "A peer-to-peer marketplace for used goods governed by community trust scores rather than anonymous ratings. Prioritises local exchange, item history transparency, and fair dispute resolution. An alternative to Willhaben and eBay that aligns incentives with long-term community health.",
    },
    {
        "quest": "Second-Hand Book and Goods Network",
        "category": "society",
        "description": "A federated platform for exchanging used books, tools, and durable goods within communities - not for profit, but for circulation. Combines donation, swap, and low-price sale in one interface. Reduces the friction between 'I no longer need this' and 'someone nearby does'.",
    },
    {
        "quest": "Car Sharing Upgrade Initiative",
        "category": "society",
        "description": "Upgrade existing car sharing systems - Zipcar-style fleets, neighbourhood cooperatives - with open APIs, interoperability between providers, and community governance models. The goal: a city where car sharing is as reliable and standardised as public transit, without platform lock-in.",
    },
    {
        "quest": "Automated Hiring System for Craftsmen",
        "category": "society",
        "description": "A platform that matches skilled craftspeople (electricians, plumbers, carpenters) with job requests using verified portfolios and transparent scheduling - without the opacity of current trade directories. Reduces the asymmetry where good craftspeople are invisible and bad ones game the reviews.",
    },
    {
        "quest": "Complete Map of the Powerful Ones",
        "category": "knowledge",
        "description": "A living, open-source knowledge graph mapping who holds real power in global systems - ownership structures, board memberships, foundation ties, revolving doors. Not a conspiracy map - a factual, citable resource built from public records. Helps anyone trace influence chains.",
    },
    {
        "quest": "Legal Quest: Biometric Unlock Under Nemo Tenetur",
        "category": "society",
        "description": "In most jurisdictions, biometric device unlocking (fingerprint, face ID) is legally distinct from password disclosure - authorities may compel the former but not the latter. The quest: litigation and legislative clarity establishing that biometric unlock is protected speech under nemo tenetur.",
    },
    {
        "quest": "Alternative to Hacker News - Globally Inclusive",
        "category": "society",
        "description": "Hacker News is the best link aggregator for technology and ideas but has a narrow demographic and no multilingual support. The quest: a federated equivalent with non-English front pages, regional moderation, and a ranking algorithm that surfaces global perspectives rather than amplifying English-language defaults.",
    },

    # ── Culture & Life ────────────────────────────────────────────────────────
    {
        "quest": "Aligned Streaming Service",
        "category": "culture",
        "description": "A music and media streaming cooperative where artists receive fair share of revenue, listeners own their listening history, and the recommendation algorithm is published and adjustable. Not Spotify - a service whose incentives are structurally aligned with the people using it.",
    },
    {
        "quest": "Self-Curated Media Feed",
        "category": "culture",
        "description": "A personal media environment where you control what enters your attention - RSS-style curation across video, audio, and text, with no algorithmic override. Includes tools to set daily time budgets per source type and a weekly digest mode that replaces the infinite scroll.",
    },
    {
        "quest": "Co-Create Visions of Meaningful Events",
        "category": "culture",
        "description": "A collaborative format for designing events that matter - gatherings, ceremonies, rituals - using structured facilitation methods. Moves event planning from logistics to meaning-making. A toolkit of practices that communities can use to create shared experiences worth remembering.",
    },
    {
        "quest": "Practical Privacy for Everyone",
        "category": "culture",
        "description": "A curated, maintained guide to reclaiming digital privacy that is realistic for non-experts - concrete steps, not exhaustive options. Covers email, phone, browser, payments, and home network. Updated quarterly as the threat landscape shifts. Free, translated, forkable.",
    },
    {
        "quest": "Inspire Others to Create",
        "category": "culture",
        "description": "A movement and practice for lowering the activation energy of making - the gap between 'I have an idea' and 'I started'. Combines mentorship matching, small-stakes publishing norms, and celebration of first attempts. The goal is more people creating, not more polished products.",
    },

    # ── Knowledge & Learning ─────────────────────────────────────────────────
    {
        "quest": "Forming Evopaydeia",
        "category": "knowledge",
        "description": "An open learning institution in the EvoBioSys tradition - where knowledge is co-created through living practice, not delivered through curriculum. Combines apprenticeship, peer dialogue, and field projects. The quest is to prototype the governance model and run the first cohort.",
    },
    {
        "quest": "EvoBioSys-Style Augmented Intelligence",
        "category": "knowledge",
        "description": "AI tools designed to amplify collective intelligence rather than replace individual thinking - surfacing connections across conversations, mapping consensus and dissent, helping groups hold more complexity. Grounded in EvoBioSys principles: the system serves life, not efficiency metrics.",
    },
    {
        "quest": "Overlap with the Mycelial Net",
        "category": "knowledge",
        "description": "The mycelial network metaphor - distributed, resilient, nourishing - applied to knowledge infrastructure. The quest: map where existing regenerative knowledge commons already function like mycelium, identify the missing connective tissue, and build bridges between nodes that don't yet know each other.",
    },

    # ── Internal to EvoBioSys ─────────────────────────────────────────────────
    {
        "quest": "interHarness",
        "category": "tools",
        "description": "An internal EvoBioSys tool: a bridge between AI agents (Claude, Gemini) and the developer terminal - enabling Claude Code to send keyboard input, read terminal output, and interact with TUI applications in a running session. Built on the ACP protocol. Enables autonomous multi-step development workflows.",
        "name": "EvoBioSys",
        "parent_project": "EvoBioSys",
    },
]

EXISTING = {
    "affirmation engine", "authentic-flirting", "bridging worldviews in politics",
    "co-ordinat.ing", "cognition-amplification through llms", "distributed substack",
    "enabling refugees as valued residents", "eu tech stack", "eu-based netlify alternative",
    "flexgmbh", "fork excalidraw", "fractal seed investment fund", "funding flow",
    "geopolitical game-theoretic machine", "green ai service", "historical library",
    "inner light network", "llm chat with open threads", "magic to learn cognitive complexity",
    "mirror eu politicians to mastodon/bluesky", "obsidian plugin", "openharness",
    "orgwide openclaw", "overnight ships between cities", "paper notes to transcription",
    "personal regenerative endowment", "playbook of creation",
    "political options mapped, autoupdated", "purpose pilot", "regenerate ukraine",
    "respectful living in old buildings", "second-hand buchhandlung", "sensemeet",
    "signal orbit / signal hub", "talk to the app to change the app",
    "the integrated llm system", "trust network tool sharing", "trustworthy servers",
    "up-to-date books",
}

submitted = 0
skipped = 0
errors = []

for q in QUESTS:
    if q["quest"].lower() in EXISTING:
        print(f"  skip (exists): {q['quest']}")
        skipped += 1
        continue

    payload = {
        "quest": q["quest"],
        "category": q["category"],
        "description": q["description"],
        "name": q.get("name", ""),
        "contact": "",
        "website": "",           # honeypot - must be empty
        "captcha_challenge": "", # old binary - no captcha check
        "captcha_nonce": "",
    }
    data = json.dumps(payload).encode()
    req = urllib.request.Request(
        API,
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            body = resp.read().decode()
            result = json.loads(body)
            print(f"  ok [{q['category']:8}] {q['quest']}")
            submitted += 1
    except urllib.error.HTTPError as e:
        body = e.read().decode()
        print(f"  ERR {e.code} {q['quest']}: {body[:80]}")
        errors.append(q["quest"])
    except Exception as e:
        print(f"  ERR {q['quest']}: {e}")
        errors.append(q["quest"])

    time.sleep(0.3)  # gentle rate spacing

print(f"\nDone. Submitted: {submitted}  Skipped (existing): {skipped}  Errors: {len(errors)}")
if errors:
    print("Failed:", errors)
