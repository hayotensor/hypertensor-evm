export const ETH_LOCAL_URL = 'http://127.0.0.1:9944'
export const SUB_LOCAL_URL = 'ws://127.0.0.1:9944'
export const SUB_PORT = 9944;
export const SS58_PREFIX = 42;
// set the tx timeout as 2 second when eable the fast-blocks feature.
export const TX_TIMEOUT = 3000;

export function stringToBinary(str: string) {
  let binaryString = '';
  for (let i = 0; i < str.length; i++) {
    const charCode = str.charCodeAt(i);
    const binaryChar = charCode.toString(2).padStart(8, '0'); 
    binaryString += binaryChar;
  }
  return binaryString;
}
