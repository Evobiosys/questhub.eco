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

ALLOWED_CATEGORIES = {"tech", "society", "finance", "knowledge", "tools", "culture"}

app = FastAPI(docs_url=None, redoc_url=None, openapi_url=None)


@app.post("/quest-submit")
async def quest_submit(
    name: str = Form(..., max_length=200),
    quest: str = Form(..., max_length=500),
    description: str = Form(..., max_length=2000),
    category: str = Form(default=""),
    email: str = Form(default=""),
):
    # Sanitise category to known values only
    safe_category = category.strip().lower()
    if safe_category not in ALLOWED_CATEGORIES:
        safe_category = ""

    submission = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "name": name.strip()[:200],
        "quest": quest.strip()[:500],
        "category": safe_category,
        "description": description.strip()[:2000],
        "email": email.strip()[:200],
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
