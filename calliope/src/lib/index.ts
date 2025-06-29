import createClient from "openapi-fetch"
import type { paths } from "./caliborn"
import { env } from "$env/dynamic/private"

export const buf2hex = (buf: ArrayBuffer) => {
    return [...new Uint8Array(buf)]
        .map(x => x.toString(16).padStart(2, '0'))
        .join('')
}

export const calibornClient = createClient<paths>({
    baseUrl: env.CALLIOPE_CALIBORN_URL
});
