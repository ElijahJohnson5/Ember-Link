import { json } from '@sveltejs/kit';
import * as jose from 'jose';
import { v4 as uuid } from 'uuid';
import type { RequestHandler } from './$types';
import { env } from '$env/dynamic/private';

export const POST: RequestHandler = async ({ request }) => {
	const { channelName } = await request.json();

	console.log(channelName);

	const privateKey = await jose.importPKCS8(env.JWT_SECRET_KEY, 'RS256');

	const token = await new jose.SignJWT({ uid: uuid() })
		.setIssuedAt()
		.setExpirationTime('30m')
		.setProtectedHeader({ alg: 'RS256' })
		.sign(privateKey);

	return json({ token });
};
