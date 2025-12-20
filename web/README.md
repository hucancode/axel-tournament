# Axel Tournament Frontend

SvelteKit-based frontend for the Axel Tournament platform.

## Features

- User authentication (email/password and Google OAuth)
- Tournament browsing and registration
- Code submission for tournaments
- Leaderboard tracking
- Admin dashboard for managing users, games, tournaments, and matches
- Profile management

## Tech Stack

- **Framework**: SvelteKit (Svelte 5)
- **Runtime**: Bun
- **Styling**: Custom CSS (no framework)
- **Type Safety**: TypeScript
- **State Management**: Svelte stores

## Development

### Prerequisites

- Bun 1.0+
- Backend API running on port 8080

### Setup

1. Install dependencies:
```bash
bun install
```

2. Copy environment variables:
```bash
cp .env.example .env
```

3. Update `.env` with your configuration:
```
PUBLIC_API_URL=http://localhost:8080
PUBLIC_GOOGLE_CLIENT_ID=your-google-client-id
```

4. Run development server:
```bash
bun run dev
```

The application will be available at http://localhost:5173

## Building for Production

```bash
bun run build
```

Preview production build:
```bash
bun run preview
```

Start production server:
```bash
bun run start
```

## Docker

Build and run with Docker:

```bash
docker build -t axel-tournament-frontend .
docker run -p 3000:3000 \
  -e PUBLIC_API_URL=http://localhost:8080 \
  axel-tournament-frontend
```

Or use docker-compose from the project root:

```bash
cd ..
docker-compose up frontend
```

## Project Structure

```
src/
├── lib/
│   ├── api.ts              # API client
│   ├── types.ts            # TypeScript types
│   ├── services/           # API service modules
│   │   ├── auth.ts
│   │   ├── tournaments.ts
│   │   ├── games.ts
│   │   ├── submissions.ts
│   │   ├── matches.ts
│   │   ├── leaderboard.ts
│   │   └── admin.ts
│   └── stores/
│       └── auth.ts         # Authentication store
├── routes/
│   ├── +layout.svelte      # Root layout
│   ├── +layout.ts          # Layout load function
│   ├── +page.svelte        # Home page
│   ├── login/              # Login page
│   ├── register/           # Registration page
│   ├── profile/            # User profile
│   ├── tournaments/        # Tournament pages
│   │   ├── +page.svelte    # List tournaments
│   │   └── [id]/           # Tournament detail & submission
│   ├── games/              # Games list
│   ├── leaderboard/        # Global leaderboard
│   └── admin/              # Admin dashboard
│       ├── users/
│       ├── games/
│       └── tournaments/
└── app.css                 # Global styles
```

## API Integration

The frontend connects to the backend API at the URL specified in `PUBLIC_API_URL`.

### Authentication

Authentication uses JWT tokens stored in localStorage. The auth store manages the current user state and token.

### Protected Routes

Pages check authentication state using the `authStore`. Admin pages additionally verify the user role is 'admin'.

## Styling

Custom CSS is used throughout the application. Main classes include:

- **Buttons**: `btn`, `btn-primary`, `btn-secondary`, `btn-danger`, `btn-success`
- **Forms**: `input`, `textarea`, `select`, `form-group`
- **Layout**: `page`, `container`, `card`, `grid`, `grid-2`, `grid-3`
- **Status badges**: `badge`, `badge-scheduled`, `badge-registration`, `badge-running`, `badge-completed`, etc.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PUBLIC_API_URL` | Backend API URL | `http://localhost:8080` |
| `PUBLIC_GOOGLE_CLIENT_ID` | Google OAuth Client ID | - |
| `PORT` | Server port (production) | `3000` |
| `HOST` | Server host (production) | `0.0.0.0` |

## License

See project root for license information.
