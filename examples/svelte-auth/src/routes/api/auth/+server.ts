import { json } from '@sveltejs/kit';
import * as jose from 'jose';
import { v4 as uuid } from 'uuid';
import type { RequestHandler } from './$types';
import { env } from '$env/dynamic/private';
import { PUBLIC_JWT_SIGNER_KEY } from '$env/static/public';

export const POST: RequestHandler = async ({ request, url }) => {
	const { channelName } = await request.json();
	const tenant = url.searchParams.get('tenant_id');
	console.log(`Channel: ${channelName}`);
	console.log(`Tenant: ${tenant}`);

	// TODO We should be looking this up in a DB based on tenant
	const privateKey = await jose.importPKCS8(env.JWT_SECRET_KEY, 'RS256');

	const token = await new jose.SignJWT({ uid: uuid() })
		.setIssuedAt()
		.setExpirationTime('30m')
		.setProtectedHeader({ alg: 'RS256' })
		.sign(privateKey);

	return json({ token });
};

export const GET: RequestHandler = async ({ url }) => {
	const tenant = url.searchParams.get('tenant_id');
	console.log(`Tenant: ${tenant}`);

	// TODO Look this up from DB based on tenant
	const publicKey = PUBLIC_JWT_SIGNER_KEY;

	return json({ publicSignerKey: publicKey });
};
