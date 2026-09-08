"""Small string helpers."""


def slugify(text):
    return text.strip().lower().replace(" ", "-")


def truncate(text, limit):
    if len(text) <= limit:
        return text
    return text[:limit] + "..."
