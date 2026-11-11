import unittest

from registry import Registry, User, load_users, parse_line


class RegistryTest(unittest.TestCase):
    def test_add_and_get_user(self):
        registry = Registry()
        user = registry.add_user(User("ada", "ada@example.org"))
        self.assertIs(registry.get_user("ada"), user)

    def test_invalid_user_is_rejected(self):
        with self.assertRaises(ValueError):
            Registry().add_user(User("", "nobody"))

    def test_remove_user(self):
        registry = load_users(["ada,ada@example.org"])
        registry.remove_user("ada")
        self.assertIsNone(registry.get_user("ada"))

    def test_find_users_by_email(self):
        registry = load_users(["ada,a@example.org", "bob,b@example.org"])
        found = registry.find_users_by_email("b@example.org")
        self.assertEqual([user.name for user in found], ["bob"])

    def test_count_active_users(self):
        registry = load_users(["ada,a@example.org", "bob,b@example.org"])
        self.assertTrue(registry.disable_user("bob"))
        self.assertFalse(registry.disable_user("eve"))
        self.assertEqual(registry.count_active_users(), 1)

    def test_get_user_names(self):
        registry = load_users(["bob,b@example.org", "", "ada,a@example.org"])
        self.assertEqual(registry.get_user_names(), ["ada", "bob"])

    def test_parse_line(self):
        user = parse_line(" ada , ada@example.org ")
        self.assertEqual((user.name, user.email), ("ada", "ada@example.org"))
        with self.assertRaises(ValueError):
            parse_line("ada")


if __name__ == "__main__":
    unittest.main()
