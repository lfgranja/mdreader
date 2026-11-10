#!/usr/bin/env python3
"""Ressoa Android device smoke: WebView load, JS module health, picker, a11y tree (CDP)."""
import json, socket, base64, os, time, select, subprocess, sys
ADB = os.path.expanduser('~/Android/Sdk/platform-tools/adb')
SERIAL = sys.argv[1] if len(sys.argv) > 1 else '192.168.100.102:39217'
PKG = 'app.mdreader'
ACT = '.MainActivity'

def adb(*a, check=True, timeout=20):
    r = subprocess.run([ADB,'-s',SERIAL]+list(a), capture_output=True, text=True, timeout=timeout)
    if check and r.returncode: raise RuntimeError((r.stdout+r.stderr).strip())
    return r.stdout

def forward_cdp():
    pid = adb('shell','pidof',PKG).strip()
    if not pid: raise RuntimeError('app not running')
    adb('forward','tcp:9222',f'localabstract:webview_devtools_remote_{pid}')
    pages = json.loads(subprocess.check_output(['curl','-s','http://127.0.0.1:9222/json'],text=True,timeout=5))
    if not pages: raise RuntimeError('no CDP pages')
    path = pages[0]['webSocketDebuggerUrl'].split('ws://127.0.0.1:9222',1)[1]
    return path

class CDP:
    def __init__(self, path):
        key=base64.b64encode(os.urandom(16)).decode(); self.s=socket.create_connection(('127.0.0.1',9222),6)
        self.s.sendall((f'GET {path} HTTP/1.1\r\nHost:127.0.0.1:9222\r\nUpgrade:websocket\r\nConnection:Upgrade\r\nSec-WebSocket-Key:{key}\r\nSec-WebSocket-Version:13\r\n\r\n').encode())
        resp=b''
        while b'\r\n\r\n' not in resp: resp+=self.s.recv(4096)
        self.mid=0; self.buf=b''; self.events=[]
    def send(self,m,p=None):
        self.mid+=1; msg=json.dumps({'id':self.mid,'method':m,'params':p or {}}).encode(); n=len(msg); hdr=bytearray([0x81]); mask=os.urandom(4)
        if n<126: hdr.append(0x80|n)
        elif n<65536: hdr.append(0x80|126); hdr+=n.to_bytes(2,'big')
        else: hdr.append(0x80|127); hdr+=n.to_bytes(8,'big')
        hdr+=mask; self.s.sendall(hdr+bytes(b^mask[i%4] for i,b in enumerate(msg))); return self.mid
    def _pump(self,timeout=0.2):
        end=time.time()+timeout
        while time.time()<end:
            r,_,_=select.select([self.s],[],[],0.15)
            if not r: continue
            b=self.s.recv(131072)
            if not b: return
            self.buf+=b
            while len(self.buf)>=2:
                op=self.buf[0]&0xf; masked=self.buf[1]&0x80; n=self.buf[1]&0x7f; i=2
                if n==126:
                    if len(self.buf)<4: return
                    n=int.from_bytes(self.buf[2:4],'big'); i=4
                elif n==127:
                    if len(self.buf)<10: return
                    n=int.from_bytes(self.buf[2:10],'big'); i=10
                mk=None
                if masked:
                    if len(self.buf)<i+4: return
                    mk=self.buf[i:i+4]; i+=4
                if len(self.buf)<i+n: return
                pl=self.buf[i:i+n]; self.buf=self.buf[i+n:]
                if mk: pl=bytes(b^mk[j%4] for j,b in enumerate(pl))
                if op==1:
                    try: self.events.append(json.loads(pl))
                    except: pass
                elif op==8: return
    def call(self,m,p=None,timeout=10):
        tid=self.send(m,p)
        end=time.time()+timeout
        while time.time()<end:
            self._pump(0.4)
            for k,e in enumerate(self.events):
                if e.get('id')==tid: self.events.pop(k); return e
        raise TimeoutError(m)
    def eval(self,expr,await_p=False,timeout=12):
        r=self.call('Runtime.evaluate',{'expression':expr,'awaitPromise':await_p,'returnByValue':True},timeout=timeout)
        if 'exceptionDetails' in r.get('result',{}): return {'__exception': r['result']['exceptionDetails']}
        return r['result']['result'].get('value')
    def close(self):
        try:self.s.close()
        except: pass

def main():
    results=[]; ok=True
    adb('shell','am','force-stop',PKG); adb('logcat','-c')
    adb('shell','am','start','-n',f'{PKG}/{ACT}')
    time.sleep(6)
    c=CDP(forward_cdp())
    try:
        c.call('Runtime.enable'); c.call('Log.enable'); c.call('Accessibility.enable',timeout=5)
        deadline=time.time()+12; health=None
        while time.time()<deadline:
            c._pump(0.4)
            try:
                h=c.eval("(function(){return {fn:typeof triggerDocumentPicker,sa:typeof sendAction,hasTitle:!!document.getElementById('titleCard'),hasTauri:!!window.__TAURI_INTERNALS__,banner:document.getElementById('reasonBanner')?.textContent||'',title:document.getElementById('documentTitle')?.textContent||'',init:!!window.__TAURI_INTERNALS__,titleAria:document.getElementById('titleCard')?.getAttribute('aria-label')||'',titleRole:document.getElementById('titleCard')?.getAttribute('role')||''}})()",timeout=4)
                health=h
                if h.get('hasTitle') and h.get('hasTauri'): break
            except Exception: pass
            time.sleep(0.3)
        exc=[e for e in c.events if e.get('method')=='Runtime.exceptionThrown']
        ax=None
        try: ax=c.call('Accessibility.getFullAXTree',{},timeout=8)
        except Exception as e: ax={'error':str(e)}
        # picker path
        adb('logcat','-c')
        pick=c.eval("(document.getElementById('titleCard')?.click(),'clicked')")
        time.sleep(2)
        act=adb('shell','dumpsys','activity','activities')
        top=[l.strip() for l in act.splitlines() if 'mResumedActivity' in l or 'topResumedActivity' in l]
        adb('shell','input','keyevent','4',check=False)
        time.sleep(1)
        log=adb('logcat','-d','-t','80',check=False)
        bad=[l for l in log.splitlines() if 'Uncaught' in l or 'SyntaxError' in l or ('TypeError' in l and 'addEventListener' in l)]
        checks={
            'no_runtime_exception_on_load': len(exc)==0,
            'title_card_a11y_label': 'selecionar documento' in ((health or {}).get('titleAria') or '').lower(),
            'title_card_present': bool((health or {}).get('hasTitle')),
            'tauri_bridge_present': bool((health or {}).get('hasTauri')),
            'title_click_opens_documentsui': any('documentsui' in t.lower() and 'PickActivity' in t for t in top),
            'no_fatal_js_logcat': len(bad)==0,
            'a11y_tree_available': 'result' in ax and 'nodes' in ax.get('result',{}),
        }
        out={'serial':SERIAL,'package':health and PKG,'lastUpdateTime': [l for l in adb('shell','dumpsys','package',PKG,check=False).splitlines() if 'lastUpdateTime' in l][:1], 'health':health,'top_after_pick':top[:1],'exceptions_on_load':len(exc),'bad_logcat':bad[:5],'a11y_nodes': (len(ax.get('result',{}).get('nodes',[])) if isinstance(ax,dict) else None), 'checks':checks}
        print(json.dumps(out,ensure_ascii=False,indent=2))
        failed=[k for k,v in checks.items() if not v]
        if failed: print('FAILED:',failed,file=sys.stderr); sys.exit(2)
        print('ALL PASS',file=sys.stderr)
    finally:
        c.close()
main()
