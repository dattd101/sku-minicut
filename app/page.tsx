'use client';
import { useEffect, useRef, useState } from 'react';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { Clapperboard, Plus, Settings2, Scissors, Download, Play, Pause, Trash2, ChevronUp, ChevronDown, RefreshCw, FolderOpen, Film, CheckCircle2, AlertCircle, LoaderCircle } from 'lucide-react';

type Tools = { ffmpeg: string; ffprobe: string; version: string };
type Clip = { id: number; path: string; name: string; duration: number; previewPath: string; hasAudio: boolean; start: number; end: number };
const fmt = (n: number) => { const v = Math.max(0, n); const h = Math.floor(v / 3600); const m = Math.floor(v % 3600 / 60); const s = Math.floor(v % 60); return `${h ? String(h).padStart(2,'0') + ':' : ''}${String(m).padStart(2,'0')}:${String(s).padStart(2,'0')}`; };
const pretty = (n: number) => `${n.toFixed(2)} giây`;
export default function Home() {
  const [tools, setTools] = useState<Tools | null>(null);
  const [toolError, setToolError] = useState('');
  const [clips, setClips] = useState<Clip[]>([]);
  const [selected, setSelected] = useState<number | null>(null);
  const [position, setPosition] = useState(0);
  const [busy, setBusy] = useState('');
  const [notice, setNotice] = useState('Sẵn sàng chỉnh sửa');
  const [settings, setSettings] = useState(false);
  const [playing, setPlaying] = useState(false);
  const player = useRef<HTMLVideoElement>(null);
  const current = clips.find(c => c.id === selected);
  const total = clips.reduce((sum, c) => sum + Math.max(0, c.end - c.start), 0);
  async function scan(folder?: string) {
    setToolError('');
    try { const found = await invoke<Tools>('detect_ffmpeg', { folder: folder || null }); setTools(found); setNotice('Đã nhận diện FFmpeg và ffprobe'); }
    catch(e) { setTools(null); setToolError(String(e)); }
  }
  useEffect(() => { void scan(); }, []);
  async function chooseFolder() {
    const result = await open({ directory: true, multiple: false, title: 'Chọn thư mục chứa ffmpeg.exe và ffprobe.exe' });
    if (typeof result === 'string') await scan(result);
  }
  async function addVideo() {
    if (!tools || busy) return;
    const paths = await open({ multiple: true, filters: [{ name: 'Video', extensions: ['mp4','mov','mkv','avi','webm','m4v'] }] });
    if (!paths) return;
    const items = Array.isArray(paths) ? paths : [paths];
    setBusy('Đang tạo preview (có thể mất thời gian với video dài)...');
    try {
      for (const path of items) {
        const data = await invoke<Omit<Clip, 'id' | 'start' | 'end'>>('import_video', { path });
        const id = Date.now() + Math.floor(Math.random() * 100000);
        setClips(old => [...old, { ...data, id, start: 0, end: data.duration }]);
        setSelected(old => old ?? id);
      }
      setNotice(`Đã thêm ${items.length} video`);
    } catch(e) { setNotice(`Lỗi: ${String(e)}`); } finally { setBusy(''); }
  }
  function selectClip(id: number) { setSelected(id); setPosition(0); setPlaying(false); }
  function updateClip(patch: Partial<Clip>) { if (selected === null) return; setClips(old => old.map(c => c.id === selected ? {...c, ...patch} : c)); }
  function move(id: number, step: number) { setClips(old => { const copy = [...old]; const index = copy.findIndex(c => c.id === id); const to = index + step; if (to < 0 || to >= copy.length) return old; [copy[index],copy[to]] = [copy[to],copy[index]]; return copy; }); }
  function seek(t: number) { setPosition(t); if (player.current) player.current.currentTime = t; }
  async function exportVideo() {
    if (!tools || !clips.length || busy) return;
    const path = await save({ defaultPath: 'minicut-output.mp4', filters: [{name:'MP4 video',extensions:['mp4']}] });
    if (!path) return;
    setBusy('Đang cắt và ghép video. Vui lòng đợi...');
    try { const result = await invoke<string>('export_video', { segments: clips.map(({path,start,end,hasAudio}) => ({path,start,end,hasAudio})), outputPath: path }); setNotice(`Xuất thành công: ${result}`); }
    catch(e) { setNotice(`Xuất thất bại: ${String(e)}`); }
    finally { setBusy(''); }
  }
  return <div className="app">
    <header className="topbar">
      <div className="brand"><div className="brand-icon"><Clapperboard size={21}/></div><div><strong>MiniCut</strong><small>VIDEO EDITOR · V3</small></div></div>
      <div className="toolbar"><button className="btn" onClick={addVideo} disabled={!tools || !!busy}><Plus size={17}/> Thêm video</button><button className="btn ghost" onClick={() => setSettings(true)}><Settings2 size={17}/> Cài đặt</button><button className="btn primary" onClick={exportVideo} disabled={!tools || !clips.length || !!busy}><Download size={17}/> Xuất MP4</button></div>
    </header>
    <div className="workspace">
      <aside className="library panel"><div className="panel-heading"><div><h2>Danh sách video</h2><p>{clips.length} clip · Kéo thứ tự bằng nút mũi tên</p></div></div>
        <button className="dropzone" onClick={addVideo} disabled={!tools || !!busy}><Plus size={22}/><strong>Thêm video</strong><span>MP4, MOV, MKV, AVI, WebM</span></button>
        <div className="clip-list">{clips.map((c,i) => <div key={c.id} className={'clip-item '+(selected === c.id ? 'active' : '')} onClick={() => selectClip(c.id)}><div className="clip-icon"><Film size={19}/></div><div className="clip-details"><strong title={c.name}>{c.name}</strong><span>{fmt(c.end-c.start)} · Clip {i+1}</span></div><div className="clip-actions"><button title="Lên" onClick={e => {e.stopPropagation();move(c.id,-1);}}><ChevronUp size={15}/></button><button title="Xuống" onClick={e => {e.stopPropagation();move(c.id,1);}}><ChevronDown size={15}/></button><button title="Xóa" onClick={e => {e.stopPropagation();setClips(old=>old.filter(x=>x.id!==c.id));if(selected===c.id){setSelected(null);setPosition(0);}}}><Trash2 size={15}/></button></div></div>)}</div>
        <div className="library-footer"><span>Tổng thời lượng</span><strong>{fmt(total)}</strong></div>
      </aside>
      <main className="main-area"><section className="preview panel"><div className="section-head"><div><h2>Xem trước video</h2><p>{current ? current.name : 'Chọn video để bắt đầu'}</p></div><span className="tag">PREVIEW</span></div>
          <div className="video-stage">{current ? <video key={current.id} ref={player} src={convertFileSrc(current.previewPath)} onTimeUpdate={e => setPosition(e.currentTarget.currentTime)} onPlay={() => setPlaying(true)} onPause={() => setPlaying(false)} onEnded={() => setPlaying(false)} playsInline /> : <div className="empty"><Film size={43}/><strong>Chưa có video</strong><span>Nhấn “Thêm video” để bắt đầu</span></div>}</div>
          <div className="playback"><button className="round-play" disabled={!current} onClick={() => { if(!player.current) return; if(player.current.paused) void player.current.play().catch(e=>setNotice(String(e))); else player.current.pause(); }}>{playing ? <Pause size={19}/> : <Play size={19}/>}</button><span className="timecode">{fmt(position)} / {fmt(current?.duration ?? 0)}</span><input className="seek" type="range" min={0} max={current?.duration ?? 1} step={0.05} value={Math.min(position,current?.duration ?? 1)} disabled={!current} onChange={e=>seek(Number(e.target.value))}/><span className="format-tag">720p xuất · 30 FPS</span></div>
        </section>
        <section className="trim panel"><div className="section-head"><div><h2><Scissors size={18}/> Cắt video</h2><p>Chọn khoảng muốn giữ lại của clip đang chọn</p></div></div>
          {current ? <div className="trim-body"><div className="trim-row"><label>Điểm bắt đầu <strong>{fmt(current.start)}</strong></label><input type="range" min={0} max={current.duration} step={0.05} value={current.start} onChange={e=>updateClip({start:Math.min(Number(e.target.value),current.end-0.05)})}/><button className="btn small" onClick={()=>updateClip({start:Math.min(position,current.end-0.05)})}>Lấy vị trí tua</button></div><div className="trim-row"><label>Điểm kết thúc <strong>{fmt(current.end)}</strong></label><input type="range" min={0} max={current.duration} step={0.05} value={current.end} onChange={e=>updateClip({end:Math.max(Number(e.target.value),current.start+0.05)})}/><button className="btn small" onClick={()=>updateClip({end:Math.max(position,current.start+0.05)})}>Lấy vị trí tua</button></div><div className="trim-summary"><CheckCircle2 size={16}/> Giữ lại {pretty(current.end-current.start)} <button onClick={()=>{updateClip({start:0,end:current.duration});seek(0);}}>Đặt lại</button></div></div> : <div className="placeholder">Chọn một clip để chỉnh sửa điểm bắt đầu và kết thúc.</div>}
        </section></main>
    </div>
    <section className="timeline"><div className="timeline-head"><div><strong>Timeline ghép video</strong><span>{clips.length} clip · {fmt(total)}</span></div><span className="timeline-hint">Nhấn vào clip để chỉnh sửa</span></div><div className="timeline-track">{clips.length ? clips.map((c,i)=><button key={c.id} className={'timeline-clip '+(c.id===selected?'selected':'')} style={{flex:Math.max(0.5,c.end-c.start)}} onClick={()=>selectClip(c.id)} title={c.name}><span>{i+1}. {c.name}</span><small>{fmt(c.end-c.start)}</small></button>) : <div className="timeline-empty">Các video sẽ xuất hiện tại đây theo thứ tự ghép</div>}</div></section>
    <footer className="status"><div className={tools?'status-ok':'status-bad'}>{tools?<CheckCircle2 size={15}/>:<AlertCircle size={15}/>} {tools?'FFmpeg đã sẵn sàng':'Chưa tìm thấy FFmpeg'}</div><span>{busy ? <><LoaderCircle className="spin" size={15}/> {busy}</> : notice}</span></footer>
    {settings && <div className="modal-backdrop" onClick={()=>setSettings(false)}><div className="modal" onClick={e=>e.stopPropagation()}><div className="modal-header"><div><h2>Cấu hình FFmpeg</h2><p>Tự động tìm FFmpeg và ffprobe trên Windows</p></div><button className="close" onClick={()=>setSettings(false)}>✕</button></div>{tools ? <><div className="tool-good"><CheckCircle2 size={19}/> Đã nhận diện FFmpeg và ffprobe</div><div className="tool-label">FFmpeg</div><div className="tool-path">{tools.ffmpeg}</div><div className="tool-label">ffprobe</div><div className="tool-path">{tools.ffprobe}</div><p className="version">{tools.version}</p></> : <div className="tool-error"><AlertCircle size={19}/>{toolError || 'Chưa phát hiện FFmpeg'}</div>}<div className="modal-actions"><button className="btn" onClick={()=>scan()}><RefreshCw size={16}/> Quét PATH</button><button className="btn primary" onClick={chooseFolder}><FolderOpen size={16}/> Chọn thư mục bin</button></div><p className="modal-tip">Ứng dụng kiểm tra PATH, thư mục cạnh EXE và các vị trí FFmpeg phổ biến. Khi chọn thủ công, hãy chọn thư mục chứa cả ffmpeg.exe và ffprobe.exe.</p></div></div>}
  </div>;
}
