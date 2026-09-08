"""Small string helpers."""


def slugify(text):
    return "-".join(text.lower().split())


def truncate(text, limit):
    if len(text) <= limit:
        return text
    return text[: limit - 3] + "..."
