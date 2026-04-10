#!/usr/bin/env python3
"""
quest-form-backend.py
=====================
Self-hosted backend for the "Share a Quest" form on questhub.eco.
Replaces Formspree with an EU-sovereign, privacy-first endpoint.

Stores submissions as append-only JSONL (no database needed).
Runs on the Infomaniak VPS behind nginx.

Deploy:
  pip install fastapi uvicorn
  uvicorn quest-form-backend:app --host 127.0.0.1 --port 8787

Or use the systemd unit quest-form.service in this directory.

Nginx proxy: see nginx-quest-form.conf in this directory.

Env vars:
  SUBMISSIONS_FILE   path to JSONL file (default: /var/local/questhub/submissions.jsonl)
  REDIRECT_SUCCESS   URL to redirect after successful submit
                     (default: https://questhub.eco/?submitted=true)
"""

import json
import os
from datetime import datetime, timezone
from fastapi import FastAPI, Form, HTTPException
from fastapi.responses import RedirectResponse

SUBMISSIONS_FILE = os.environ.get(
    "SUBMISSIONS_FILE",
    "/var/local/questhub/submissions.jsonl",
)
REDIRECT_SUCCESS = os.environ.get(
    "REDIRECT_SUCCESS",
    "https://questhub.eco/?submitted=true",
)

CURATED_CATEGORIES = {"tech", "society", "finance", "knowledge", "tools", "culture"}

app = FastAPI(docs_url=None, redoc_url=None, openapi_url=None)


@app.post("/quest-submit")
async def quest_submit(
    name: str = Form(..., max_length=200),
    quest: str = Form(..., max_length=500),
    description: str = Form(..., max_length=2000),
    category: str = Form(default=""),
    category_custom: str = Form(default=""),
    contact: str = Form(default=""),
):
    raw_cat = category.strip().lower()
    if raw_cat in CURATED_CATEGORIES:
        final_category = raw_cat
        category_type = "curated"
    elif raw_cat == "other" and category_custom.strip():
        final_category = category_custom.strip()[:80]
        category_type = "community"
    else:
        final_category = raw_cat
        category_type = "curated" if raw_cat in CURATED_CATEGORIES else "community"

    submission = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "name": name.strip()[:200],
        "quest": quest.strip()[:500],
        "category": final_category,
        "category_type": category_type,
        "description": description.strip()[:2000],
        "contact": contact.strip()[:300],
    }

    # Ensure directory exists
    os.makedirs(os.path.dirname(SUBMISSIONS_FILE), exist_ok=True)

    # Append-only write — atomic enough for low volume
    with open(SUBMISSIONS_FILE, "a", encoding="utf-8") as f:
        f.write(json.dumps(submission, ensure_ascii=False) + "\n")

    # 303 See Other — tells browser to GET the redirect URL (standard post-form pattern)
    return RedirectResponse(REDIRECT_SUCCESS, status_code=303)


@app.get("/health")
async def health():
    return {"status": "ok"}
