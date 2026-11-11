"""The registry stores users."""


class User:
    """A user has a name and an email."""

    def __init__(self, name, email):
        self.name = name
        self.email = email
        self.active = True

    def is_valid(self):
        """The user has a valid email."""
        return bool(self.name) and "@" in self.email


class Registry:
    """The registry stores users."""

    def __init__(self):
        self.users = {}

    def add_user(self, user):
        if not user.is_valid():
            raise ValueError("invalid user")
        self.users[user.name] = user
        return user

    def get_user(self, name):
        return self.users.get(name)

    def remove_user(self, name):
        return self.users.pop(name, None)

    def find_users_by_email(self, email):
        return [user for user in self.users.values() if user.email == email]

    def count_active_users(self):
        return sum(1 for user in self.users.values() if user.active)

    def get_user_names(self):
        return sorted(self.users)

    def disable_user(self, name):
        user = self.get_user(name)
        if user is None:
            return False
        user.active = False
        return True


def parse_line(line):
    """The line gives a user."""
    fields = tuple(item.strip() for item in line.split(","))
    if len(fields) != 2:
        raise ValueError("expected name,email")
    name, email = fields
    return User(name, email)


def load_users(lines):
    registry = Registry()
    # We read every line.
    for line in lines:
        if line.strip():
            registry.add_user(parse_line(line))
    return registry
