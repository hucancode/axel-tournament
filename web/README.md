# Axel Tournament

Frontend for players and admins.

## Features
- Sign-in, registration, and profile management
- Tournament discovery and registration
- Code submission and match results viewing
- Leaderboards
- Admin dashboard for users, games, tournaments, and matches

## Get started
Run `bun install` if you haven't done so.
```bash
# cp .env.example .env
bun run dev
```
Open a storybook session to test your UI components
```bash
bun run storybook
```

## Environment variables
- `PUBLIC_API_URL`: backend API base URL, default to the default URL of your local API server
