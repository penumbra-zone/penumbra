# Running a frontend

In order to use the [Prax] wallet web extension, users must visit a trusted
website that leverages the web extension in order to interact with the [Penumbra] network.
While users can choose to grant access to website run by a party they trust,
this guide demonstrates how a user can self-host a frontend for use by themselves and others.

## About minifront

[Minifront] is minimal frontend for interacting with the [Penumbra] chain.
A number of technical decisions were made to ensure minifront is maximally client side and does not leak
information unnecessarily:

- Client-side biased js framework ✅ Hash routing ✅
- Pre-load all static assets ⚠️ (in progress...)
- Server rendering ❌
- Route-based code splitting ❌
- Idiomatic urls & query params ❌
- Build-time pre-rendering ❌

[Read more](https://x.com/grod220/status/1760217326245285923) about how this frontend embraces censorship resistance and
privacy.

## Deploy anywhere

### Automatically w/ github action

The [minifront deployer repo](https://github.com/penumbra-zone/minifront-deployer) has a github action
that can manage minifront's the build+deployment steps on a schedule. Using this will allow you to
always host the latest code commited to [@penumbra-zone/web](https://github.com/penumbra-zone/web).
Simply fork, add environment variables to your repo, and make customizations for your particular host.

### Manual deploys

The `dist/` output of the build is simply static assets. That means, it basically can be hosted anywhere.
First, download `dist.zip` from
the [latest minifront release from github](https://github.com/penumbra-zone/web/releases?q=minifront&expanded=true).
Unzip that and take it to a variety of host providers. Examples:

### Vercel

```shell
npm i -g vercel # install the Vercel cli
vercel login
vercel ./dist
```

### Netlify

```shell
npm install netlify-cli -g # install the netlify cli
netlify login
cd ./dist
netlify deploy
```

### Github pages

Can follow [this guide](https://pages.github.com/).
Let's say your username is **roboto7237**.
First create a new repo specifically named in this format: roboto7237.github.io. Then do:

```shell
git clone https://github.com/roboto7237/roboto7237.github.io
cp -r ./dist/* ./roboto7237.github.io/ # copies all minifront code into the new folder
git add --all
git commit -m "Initial commit"
git push -u origin main
```

### Alternative SaaS providers

There are a ton of other places where static files can be hosted:

- [Cloudflare pages](https://pages.cloudflare.com/)
- [Firebase](https://firebase.google.com/docs/hosting)
- [Render](https://render.com/)
- [Surge](https://surge.sh/)
- [Google cloud](https://cloud.google.com/storage/docs/hosting-static-website)
- [AWS](https://docs.aws.amazon.com/AmazonS3/latest/userguide/WebsiteHosting.html)

## Local build

If you want to run the minifront web interface from your local computer, you'll
need a few development tools:

- Install [nodejs](https://nodejs.org/)
- Install [pnpm](https://pnpm.io/installation)
- Add buf registry: `npm config set @buf:registry https://buf.build/gen/npm/v1/`

Then clone the [Minifront] git repo and run:

```shell
pnpm install
pnpm dev
# Will be live at https://localhost:5173/
```

## Frontend embedded in fullnode

If you're already [running a fullnode](./running-node.md), then you don't need to do anything else:
a bundled version of the frontend code is available at `https://<YOUR_NODE_URL>/app`. Simply navigate
to that site after installing [Prax], and authorize the web extension to connect to it.

[Minifront]: https://github.com/penumbra-zone/web/tree/main/apps/minifront

[Prax]: https://chromewebstore.google.com/detail/prax-wallet/lkpmkhpnhknhmibgnmmhdhgdilepfghe

[Penumbra]: https://penumbra.zone
