"""Small calendar helpers."""


def is_leap(year):
    return year % 4 == 0 and (year % 100 != 0 or year % 400 == 0)


def quarter(month):
    return (month + 2) // 3
