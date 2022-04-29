const { addressToScript, serializeScript  } = require('@nervosnetwork/ckb-sdk-utils')
const { Collector, Aggregator, generateMintCotaTx } = require("@nervina-labs/cota-sdk");

const ISSUER_PRIVATE_KEY = '0xc5bd09c9b954559c70a77d68bde95369e2ce910556ddc20f739080cde3b62ef2'
const ISSUER_ADDRESS = 'ckt1qyq0scej4vn0uka238m63azcel7cmcme7f2sxj5ska'
const RECEIVER1_ADDRESS = 'ckt1qyqrq7vdeh5a8rnp4n2tuuu08p5uw8a5qdtqrpvdsg'
const RECEIVER2_ADDRESS = 'ckt1qyqz8vxeyrv4nur4j27ktp34fmwnua9wuyqqggd748'

const secp256k1CellDep = async (ckb) => {
  const secp256k1Dep = (await ckb.loadDeps()).secp256k1Dep
  return { outPoint: secp256k1Dep.outPoint, depType: 'depGroup' }
}

const run = async () => {
  const service = {
    collector: new Collector({ ckbNodeUrl: 'http://localhost:8114', ckbIndexerUrl: 'http://localhost:8116' }),
    aggregator: new Aggregator({ registryUrl: 'http://localhost:3050', cotaUrl: 'http://localhost:3030' }),
  }
  const ckb = service.collector.getCkb()
  const mintLock = addressToScript(ISSUER_ADDRESS)

  const mintCotaInfo = {
    cotaId: '0x096b5d210b3b32fab6f8fbd937e21b06b5d91e86',
    withdrawals: [
      {
        state: '0x00',
        characteristic: '0x0505050505050505050505050505050505050505',
        toLockScript: serializeScript(addressToScript(RECEIVER1_ADDRESS)),
      },
      {
        state: '0x00',
        characteristic: '0x0505050505050505050505050505050505050505',
        toLockScript: serializeScript(addressToScript(RECEIVER2_ADDRESS)),
      },
    ],
  }
  let rawTx = await generateMintCotaTx(service, mintLock, mintCotaInfo)

  const secp256k1Dep = await secp256k1CellDep(ckb)
  rawTx.cellDeps.push(secp256k1Dep)

  const signedTx = ckb.signTransaction(ISSUER_PRIVATE_KEY)(rawTx)
  console.log(JSON.stringify(signedTx))
  let txHash = await ckb.rpc.sendTransaction(signedTx, 'passthrough')
  console.info(`Mint cota nft tx has been sent with tx hash ${txHash}`)
}

run()

