"""Small string helpers."""


def slugify(text):
    return "-".join(text.strip().lower().split())


def truncate(text, limit):
    if len(text) <= limit:
        return text
    return text[: max(limit - 3, 0)] + "..."
