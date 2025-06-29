import { calibornClient } from "$lib";
import { error, redirect, type RequestHandler } from "@sveltejs/kit";

export const GET: RequestHandler = async ({ cookies, url, fetch }) => {
    const userState = cookies.get('login_state');
    const discordState = url.searchParams.get('state');
    const code = url.searchParams.get('code');

    if (!userState || !discordState || userState !== discordState) {
        error(400, "State verification failed");
    }

    cookies.delete('login_state', { path: '/' })

    if (!code) {
        error(400, "No code provided");
    }

    const { data, error: calibornError } = await calibornClient.POST("/auth/discord/login", {
        body: {
            code
        },
        fetch
    });

    if (calibornError) {
        error(401, "Failed to login: " + calibornError.message);
    }

    console.log(data);
    cookies.set('token', data.token, { path: '/' })

    redirect(302, '/');
}