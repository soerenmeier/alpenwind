
// key should be encoded
async function deriveKey(key, salt) {
	// convert to bytes
	key = await crypto.subtle.importKey(
		'raw', key, 'PBKDF2', false,
		['deriveKey']
	);
	return await crypto.subtle.deriveKey(
		{ name: 'PBKDF2', hash: 'SHA-256', salt, iterations: 12 },
		key,
		{ name: 'AES-GCM', length: 256 },
		false,
		['encrypt', 'decrypt']
	);
}

async function base64encode(bytes) {
	const base64url = await new Promise(res => {
		const reader = new FileReader;
		reader.onload = () => res(reader.result);
		reader.readAsDataURL(new Blob([bytes]));
	});

	return base64url.split(',', 2)[1];
}

async function base64decode(str) {
	const base64url = 'data:application/octet-binary;base64,' + str;

	const b = await fetch(base64url).then(r => r.arrayBuffer());
	return new Uint8Array(b);
}

export async function encrypt(key, data) {
	const encoder = new TextEncoder;
	key = encoder.encode(key);
	const iv = crypto.getRandomValues(new Uint8Array(16));
	key = await deriveKey(key, iv);

	data = encoder.encode(data);

	data = await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, data);
	data = new Uint8Array(data);
	const finalData = new Uint8Array(iv.length + data.length);
	finalData.set(iv);
	finalData.set(data, iv.length);

	return await base64encode(finalData);
}

export async function decrypt(key, data) {
	data = await base64decode(data);
	const iv = data.slice(0, 16);
	data = data.slice(16);

	const encoder = new TextEncoder;
	key = encoder.encode(key);

	key = await deriveKey(key, iv);

	data = await crypto.subtle.decrypt({ name: 'AES-GCM', iv }, key, data);

	const decoder = new TextDecoder;
	return decoder.decode(data);
}