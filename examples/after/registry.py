"""Fa tupapbivur ad-det-registeret brukarar."""


class Brukar:
    """Fa ʿinda brukar namn wa e-post."""

    def __init__(self, namn, e_post):
        self.namn = namn
        self.e_post = e_post
        self.hal_aktiv = True

    def hal_gyldig(self):
        """Fa ʿinda ad-den-brukaren e-post gyldig."""
        return bool(self.namn) and "@" in self.e_post


class Register:
    """Fa tupapbivur ad-det-registeret brukarar."""

    def __init__(self):
        self.brukarar = {}

    def utluf_ad_den_brukaren(self, brukar):
        if not brukar.hal_gyldig():
            raise ValueError("invalid user")
        self.brukarar[brukar.namn] = brukar
        return brukar

    def uhnut_ad_den_brukaren(self, namn):
        return self.brukarar.get(namn)

    def fajrin_ad_den_brukaren(self, namn):
        return self.brukarar.pop(namn, None)

    def ufnun_ad_dei_brukarane_7asaba_ad_den_e_posten(self, e_post):
        return [brukar for brukar in self.brukarar.values() if brukar.e_post == e_post]

    def utlul_ad_dei_brukarane_al_aktive(self):
        return sum(1 for brukar in self.brukarar.values() if brukar.hal_aktiv)

    def uhnut_namn_ad_den_brukaren(self):
        return sorted(self.brukarar)

    def saprir_ad_den_brukaren(self, namn):
        brukar = self.uhnut_ad_den_brukaren(namn)
        if brukar is None:
            return False
        brukar.hal_aktiv = False
        return True


def utluk_ad_den_linja(linje):
    """Fa tagjuwur ad-den-linja brukar."""
    feltaan = tuple(element.strip() for element in linje.split(","))
    if len(feltaan) != 2:
        raise ValueError("expected name,email")
    namn, e_post = feltaan
    return Brukar(namn, e_post)


def ulsut_ad_dei_brukarane(linjer):
    register = Register()
    # Fa nalsuwur vi kull linje.
    for linje in linjer:
        if linje.strip():
            register.utluf_ad_den_brukaren(utluk_ad_den_linja(linje))
    return register
