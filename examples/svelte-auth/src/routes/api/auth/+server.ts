import { json } from '@sveltejs/kit';
import { UnsecuredJWT } from 'jose';
import { v4 as uuid } from 'uuid';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request }) => {
	const { channelName } = await request.json();

	console.log(channelName);

	const token = new UnsecuredJWT({ uid: uuid() }).setIssuedAt().setExpirationTime('30m').encode();

	return json({ token });
};
