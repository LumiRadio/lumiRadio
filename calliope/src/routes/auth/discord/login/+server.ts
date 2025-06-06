import { buf2hex } from "$lib";
import { redirect, type RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";

export const GET: RequestHandler = async ({ cookies, url }) => {
    const state = buf2hex(crypto.getRandomValues(new Uint8Array(16)).buffer);

    cookies.set('login_state', state, {
        path: '/',
        httpOnly: true,
        secure: true,
        sameSite: 'lax',
        maxAge: 600
    });

    const clientId = env.CALLIOPE_DISCORD_CLIENT_ID;
    console.log(url.origin);
    const redirectUri = `${url.origin}/auth/discord/callback`;

    const authUrl = new URL("https://discord.com/oauth2/authorize");
    authUrl.searchParams.append("client_id", clientId);
    authUrl.searchParams.append("redirect_uri", redirectUri);
    authUrl.searchParams.append("response_type", "code");
    authUrl.searchParams.append("state", state);
    authUrl.searchParams.append("scope", "identify connections");

    throw redirect(302, authUrl.toString());
}