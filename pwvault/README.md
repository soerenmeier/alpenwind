
## Export script from ctrl pwvault
```js
const jsonString = '';
const masterPw = '';

const json = JSON.parse(jsonString);
const passwords = json.map(pw => {
	return {
		site: pw.site,
		username: pw.username,
		password: CryptoJS.AES.decrypt(pw.pw, masterPw).toString(CryptoJS.enc.Utf8)
	};
});
console.log(JSON.stringify(passwords));
```
## Import script
```js
const jsonString = '';
const masterPw = '';

const json = JSON.parse(jsonString);
(async () => {
	for (let pw of json) {
		await window.dbgPwImport({
			id: null,
			site: pw.site,
			domain: '-',
			username: pw.username,
			password: pw.password
		}, masterPw);
	}
})();
```