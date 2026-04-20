#!/usr/bin/env python3
"""Extract the 37 hardcoded quest cards from questhub.eco/index.html
and generate a Kidur-compatible seed .jsonl file.

Output format: one LogEntry per line, matching kidur-log serialization:
  {"seq":N,"ts":"...","op":"create_node","node":{...}}
"""

import json
import uuid
import sys
from datetime import datetime, timezone

# Seed quest data extracted from questhub.eco/index.html
# Format: (title, category, lifecycle, description, parent_project)
# lifecycle mapping: gold=sprouting, green=growing, dormant=identified

QUESTS = [
    # ---- Technology & Infrastructure (10) ----
    ("Signal Orbit / Signal Hub", "tech", "sprouting",
     "General communications asset management workflow \u2014 unified signal routing across sovereign infrastructure.",
     "idea2.life"),
    ("Trustworthy Servers", "tech", "growing",
     "Trusted hosting infrastructure where sovereignty is the default, not the exception. EU-first, open-source, auditable.",
     None),
    ("Orgwide OpenClaw", "tech", "sprouting",
     "Organization-wide AI assistant \u2014 a shared intelligence layer that respects boundaries and amplifies collective capability.",
     None),
    ("EU Tech Stack", "tech", "growing",
     "Sovereign European technology \u2014 mapping, curating, and building a complete stack free from non-EU jurisdictional risk.",
     None),
    ("EU-based Netlify Alternative", "tech", "sprouting",
     "Self-hosted deployment platform \u2014 because where your code ships from matters as much as the code itself.",
     None),
    ("The Integrated LLM System", "tech", "sprouting",
     "A personal AI that knows your context, respects your data, and grows with you \u2014 not against you.",
     None),
    ("Talk to the App to Change the App", "tech", "identified",
     "Meta-programming through conversation \u2014 software that reshapes itself when you describe what you need.",
     None),
    ("LLM Chat with Open Threads", "tech", "identified",
     "A chat experience where conversations branch, reconnect, and persist \u2014 threads that stay alive across sessions.",
     None),
    ("Fork Excalidraw", "tech", "identified",
     "A sovereign whiteboard tool \u2014 visual thinking infrastructure that lives on your terms.",
     None),
    ("Obsidian Plugin", "tech", "identified",
     "Bridging personal knowledge management with the broader EvoBioSys ecosystem through plugin development.",
     None),

    # ---- Society & Politics (6) ----
    ("Bridging Worldviews in Politics", "society", "growing",
     "Finding synergy and omni-win solutions across political divides \u2014 because the best answer usually lives between the camps.",
     None),
    ("Political Options Mapped, Autoupdated", "society", "sprouting",
     "Political intelligence that stays current \u2014 a living map of positions, proposals, and possibilities across the spectrum.",
     None),
    ("Geopolitical Game-Theoretic Machine", "society", "identified",
     "Strategy simulation for geopolitics \u2014 modeling interdependencies, incentives, and outcomes at the systems level.",
     None),
    ("Regenerate Ukraine", "society", "growing",
     "Regenerative reconstruction \u2014 rebuilding that goes beyond restoration to create something stronger and more resilient.",
     None),
    ("Enabling Refugees as Valued Residents", "society", "sprouting",
     "Pathways that recognize refugees as contributors from day one \u2014 not burdens to be managed but potential to be unlocked.",
     None),
    ("Mirror EU Politicians to Mastodon/Bluesky", "society", "identified",
     "Freeing political communication from Twitter \u2014 mirroring EU politicians onto open, decentralized platforms.",
     None),

    # ---- Finance & Economics (4) ----
    ("Personal Regenerative Endowment", "finance", "growing",
     "Sovereign wealth for individuals \u2014 a personal endowment model that compounds purpose alongside capital.",
     None),
    ("Funding Flow", "finance", "sprouting",
     "Capital flow management \u2014 making money move with intention, transparency, and alignment to regenerative outcomes.",
     None),
    ("flexGmbH", "finance", "identified",
     "A flexible company structure \u2014 reimagining the GmbH for fluid, purpose-driven ventures that evolve over time.",
     None),
    ("Fractal Seed Investment Fund", "finance", "sprouting",
     "Investment at every scale \u2014 fractal funding that flows from micro-grants to major capital through one coherent model.",
     "SoFin"),

    # ---- Knowledge & Learning (5) ----
    ("Cognition-Amplification through LLMs", "knowledge", "sprouting",
     "Using language models not as replacements for thinking but as amplifiers \u2014 extending cognitive reach without losing depth.",
     None),
    ("Historical Library", "knowledge", "identified",
     "A curated collection that bridges past and present \u2014 historical knowledge made accessible and searchable.",
     None),
    ("Magic to Learn Cognitive Complexity", "knowledge", "identified",
     "Using the structure and surprise of magic to teach systems thinking and cognitive flexibility.",
     None),
    ("Playbook of Creation", "knowledge", "sprouting",
     "Patterns, recipes, and principles for bringing ideas into existence \u2014 a living playbook for makers and builders.",
     None),
    ("Up-to-Date Books", "knowledge", "growing",
     "Books that evolve \u2014 living publications that update as knowledge grows, rather than freezing at the moment of printing.",
     None),

    # ---- Tools & Products (8) ----
    ("Purpose Pilot", "tools", "growing",
     "A purpose discovery tool \u2014 helping people find and articulate what they are here to do, with structure and honesty.",
     None),
    ("Co-ordinat.ing", "tools", "sprouting",
     "A coordination platform \u2014 making it possible for groups to move together without a central command-and-control layer.",
     None),
    ("Paper Notes to Transcription", "tools", "identified",
     "Bridging analog and digital \u2014 turning handwritten notes into structured, searchable, connected text.",
     None),
    ("Affirmation Engine", "tools", "identified",
     "Intentional self-reinforcement \u2014 a tool that delivers the right affirmation at the right moment, not random positivity.",
     None),
    ("SenseMeet", "tools", "sprouting",
     "A meeting tool built for sense-making \u2014 where meetings produce shared understanding, not just action items.",
     "In-Flow"),
    ("Distributed Substack", "tools", "sprouting",
     "Decentralized publishing \u2014 the newsletter model without the platform dependency. Your audience, your infrastructure.",
     None),
    ("Trust Network Tool Sharing", "tools", "identified",
     "Sharing tools and resources within trusted networks \u2014 collaborative access without the overhead of formal institutions.",
     None),
    ("Green AI Service", "tools", "growing",
     "AI infrastructure that accounts for its environmental cost \u2014 compute that is powerful and responsible.",
     None),

    # ---- Culture & Life (4) ----
    ("Authentic-Flirting", "culture", "sprouting",
     "The art and practice of genuine human connection \u2014 playful, honest, embodied presence without performance.",
     None),
    ("Second-hand Buchhandlung", "culture", "identified",
     "A second-hand bookshop as cultural infrastructure \u2014 where books find new readers and readers find community.",
     None),
    ("Overnight Ships Between Cities", "culture", "identified",
     "Coastal and river-city connections via overnight ships \u2014 slow travel as infrastructure, not just nostalgia.",
     None),
    ("Respectful Living in Old Buildings", "culture", "growing",
     "Inhabiting historic structures with care \u2014 practices that honor what was built while adapting it for living now.",
     None),
]


def make_field(kind: str, value) -> dict:
    """Create a Kidur FieldValue JSON object."""
    return {"kind": kind, "value": value}


def make_log_entry(seq: int, title: str, category: str, lifecycle: str,
                   description: str, parent_project: str | None) -> dict:
    """Create a Kidur LogEntry JSON object."""
    ts = datetime(2026, 4, 13, 12, 0, 0, tzinfo=timezone.utc).isoformat()
    node_id = str(uuid.uuid4())

    fields = {
        "status": make_field("enum", "active"),
        "category": make_field("enum", category),
        "description": make_field("rich_text", description),
        "lifecycle_stage": make_field("enum", lifecycle),
        "submitter_name": make_field("text", "Jakob Possert-Bienzle"),
    }

    if parent_project:
        fields["parent_project"] = make_field("text", parent_project)

    return {
        "seq": seq,
        "ts": ts,
        "op": "create_node",
        "node": {
            "id": node_id,
            "parent_id": None,
            "sort_order": 0.0,
            "content": title,
            "supertag": "quest",
            "fields": fields,
            "created_at": ts,
            "updated_at": ts,
            "created_by": "seed",
            "visibility": "public",
        },
    }


def main():
    output = sys.argv[1] if len(sys.argv) > 1 else "seeds/questhub-quests.jsonl"
    entries = []
    for i, (title, cat, lifecycle, desc, parent) in enumerate(QUESTS, start=1):
        entry = make_log_entry(i, title, cat, lifecycle, desc, parent)
        entries.append(json.dumps(entry, ensure_ascii=False))

    with open(output, "w") as f:
        f.write("\n".join(entries) + "\n")

    print(f"Wrote {len(entries)} seed quests to {output}")


if __name__ == "__main__":
    main()
