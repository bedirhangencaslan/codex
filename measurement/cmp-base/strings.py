"""Small string helpers."""
import re


def slugify(text):
    return re.sub(r"\s+", "-", text.strip().lower())


def truncate(text, limit):
    if len(text) <= limit:
        return text
    return text[:limit - 3] + "..."
