"""Small string helpers."""


def slugify(text):
    return " ".join(text.split()).lower().replace(" ", "-")


def truncate(text, limit):
    if len(text) <= limit:
        return text
    if limit < len("..."):
        raise ValueError("limit must be at least 3")
    return text[: limit - len("...")] + "..."
