import unittest

from registry import Register, Brukar, ulsut_ad_dei_brukarane, utluk_ad_den_linja


class TestAdDetRegisteret(unittest.TestCase):
    def test_utluf_wa_uhnut_ad_den_brukaren(self):
        register = Register()
        brukar = register.utluf_ad_den_brukaren(Brukar("ada", "ada@example.org"))
        self.assertIs(register.uhnut_ad_den_brukaren("ada"), brukar)

    def test_ad_den_brukaren_al_ugyldige_al_mavvuuse(self):
        with self.assertRaises(ValueError):
            Register().utluf_ad_den_brukaren(Brukar("", "nobody"))

    def test_fajrin_ad_den_brukaren(self):
        register = ulsut_ad_dei_brukarane(["ada,ada@example.org"])
        register.fajrin_ad_den_brukaren("ada")
        self.assertIsNone(register.uhnut_ad_den_brukaren("ada"))

    def test_ufnun_ad_dei_brukarane_7asaba_ad_den_e_posten(self):
        register = ulsut_ad_dei_brukarane(["ada,a@example.org", "bob,b@example.org"])
        mafnuun = register.ufnun_ad_dei_brukarane_7asaba_ad_den_e_posten("b@example.org")
        self.assertEqual([brukar.namn for brukar in mafnuun], ["bob"])

    def test_utlul_ad_dei_brukarane_al_aktive(self):
        register = ulsut_ad_dei_brukarane(["ada,a@example.org", "bob,b@example.org"])
        self.assertTrue(register.saprir_ad_den_brukaren("bob"))
        self.assertFalse(register.saprir_ad_den_brukaren("eve"))
        self.assertEqual(register.utlul_ad_dei_brukarane_al_aktive(), 1)

    def test_uhnut_namn_ad_den_brukaren(self):
        register = ulsut_ad_dei_brukarane(["bob,b@example.org", "", "ada,a@example.org"])
        self.assertEqual(register.uhnut_namn_ad_den_brukaren(), ["ada", "bob"])

    def test_utluk_ad_den_linja(self):
        brukar = utluk_ad_den_linja(" ada , ada@example.org ")
        self.assertEqual((brukar.namn, brukar.e_post), ("ada", "ada@example.org"))
        with self.assertRaises(ValueError):
            utluk_ad_den_linja("ada")


if __name__ == "__main__":
    unittest.main()
