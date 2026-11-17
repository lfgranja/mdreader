#!/usr/bin/env python3
"""Ressoa Android accessibility smoke (no privileged secure-settings changes).

Checks: app process, WebView CDP Accessibility.getFullAXTree semantics,
uiautomator hierarchy dump for content-desc/text/clickable labels, and reports
TalkBack service state. Enabling TalkBack requires explicit user/device authorization
when ADB secure settings are restricted.
"""
import json, socket, base64, os, time, select, subprocess, sys, xml.etree.ElementTree as ET
ADB = os.path.expanduser('~/Android/Sdk/platform-tools/adb')
SERIAL = sys.argv[1] if len(sys.argv) > 1 else '192.168.100.102:39217'
PKG = 'app.mdreader'

def adb(*a, check=True, timeout=20):
    r = subprocess.run([ADB,'-s',SERIAL]+list(a), capture_output=True, text=True, timeout=timeout)
    if check and r.returncode not in (0,): raise RuntimeError((r.stdout+r.stderr).strip())
    return (r.stdout or '') + (r.stderr or '')

def forward_cdp():
    pid = adb('shell','pidof',PKG).strip().splitlines()[0].strip()
    adb('forward','tcp:9222',f'localabstract:webview_devtools_remote_{pid}')
    pages = json.loads(subprocess.check_output(['curl','-s','http://127.0.0.1:9222/json'],text=True,timeout=5))
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
        tid=self.send(m,p); end=time.time()+timeout
        while time.time()<end:
            self._pump(0.4)
            for k,e in enumerate(self.events):
                if e.get('id')==tid: self.events.pop(k); return e
        raise TimeoutError(m)
    def close(self):
        try:self.s.close()
        except: pass

adb('shell','am','force-stop',PKG); adb('shell','am','start','-n',f'{PKG}/.MainActivity'); time.sleep(6)
svc=adb('shell','settings','get','secure','enabled_accessibility_services',check=False).strip()
talkback='talkback' in svc.lower() if svc and svc!='null' else False
c=CDP(forward_cdp())
try:
    c.call('Accessibility.enable',{},timeout=8)
    tree=c.call('Accessibility.getFullAXTree',{},timeout=12).get('result',{}).get('nodes',[])
    names=[]
    for n in tree:
        nm=(n.get('name') or {}).get('value')
        role=(n.get('role') or {}).get('value')
        if nm and role in ('button','link','heading','generic','textbox','img','region','RootWebArea','WebArea'):
            names.append({'role':role,'name':nm[:120]})
finally: c.close()
adb('shell','uiautomator','dump','/sdcard/ui.xml',check=False); time.sleep(1)
xml=adb('shell','cat','/sdcard/ui.xml',check=False)
ui_items=[]
try:
    root=ET.fromstring(xml)
    for node in root.iter('node'):
        d={k:node.attrib.get(k,'') for k in ('text','content-desc','class','clickable','enabled','resource-id','bounds')}
        if d['text'] or d['content-desc'] or d['clickable']=='true': ui_items.append(d)
except Exception as e: ui_items=[{'parse_error':str(e)}]
checks={
  'app_running': bool(adb('shell','pidof',PKG,check=False).strip()),
  'cdp_a11y_tree_nonempty': len(tree)>5,
  'title_picker_name_present': any('selecionar documento' in (n['name'] or '').lower() for n in names),
  'uiautomator_dump_ok': bool(ui_items) and 'parse_error' not in ui_items[0],
  'talkback_enabled': talkback,
}
out={'serial':SERIAL,'enabled_accessibility_services':svc,'talkback_enabled':talkback,'ax_nodes':len(tree),'named_ax':names[:25],'ui_items':ui_items[:25],'checks':checks,
     'notes':['TalkBack automation enablement requires explicit device/user authorization if secure settings are restricted; this check is observational and does not change secure settings.']}
print(json.dumps(out,ensure_ascii=False,indent=2))
failed=[k for k,v in checks.items() if not v and k!='talkback_enabled']
if failed: print('FAILED:',failed,file=sys.stderr); sys.exit(2)
if not talkback: print('PASS (observational). TalkBack not enabled; enablement needs explicit authorization/capability.',file=sys.stderr)
else: print('ALL PASS',file=sys.stderr)
