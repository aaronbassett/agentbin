class AgentbinBadge extends HTMLElement {
    constructor() {
        super();
        this.attachShadow({ mode: 'open' });
    }

    connectedCallback() {
        const meta = JSON.parse(this.getAttribute('data-meta') || '{}');

        this.shadowRoot.innerHTML = `
            <style>
                :host {
                    position: fixed;
                    bottom: 20px;
                    right: 20px;
                    z-index: 999999;
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
                }
                .badge-btn {
                    width: 40px;
                    height: 40px;
                    border-radius: 50%;
                    background: #1a1a2e;
                    color: #fff;
                    border: none;
                    cursor: pointer;
                    font-size: 18px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    box-shadow: 0 2px 8px rgba(0,0,0,0.2);
                    transition: transform 0.2s;
                }
                .badge-btn:hover { transform: scale(1.1); }
                .popover {
                    display: none;
                    position: absolute;
                    bottom: 50px;
                    right: 0;
                    width: 320px;
                    background: #fff;
                    border-radius: 8px;
                    box-shadow: 0 4px 20px rgba(0,0,0,0.15);
                    padding: 16px;
                    font-size: 14px;
                    color: #333;
                    max-height: 400px;
                    overflow-y: auto;
                }
                .popover.open { display: block; }
                .popover h3 { margin: 0 0 4px; font-size: 16px; }
                .popover .subtitle { color: #666; margin: 0 0 8px; font-size: 13px; }
                .popover .field { margin: 6px 0; line-height: 1.4; }
                .popover .label { font-weight: 600; color: #555; }
                .popover .tags { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 4px; }
                .popover .tag {
                    background: #e8e8e8;
                    padding: 2px 8px;
                    border-radius: 12px;
                    font-size: 12px;
                }
                .popover .separator { border: none; border-top: 1px solid #eee; margin: 10px 0; }
                .popover a { color: #0066cc; text-decoration: none; }
                .popover a:hover { text-decoration: underline; }
            </style>
            <button class="badge-btn" title="File info">&#8505;</button>
            <div class="popover" id="popover"></div>
        `;

        const popover = this.shadowRoot.getElementById('popover');
        popover.innerHTML = this._buildPopoverContent(meta);

        this.shadowRoot.querySelector('.badge-btn').addEventListener('click', () => {
            popover.classList.toggle('open');
        });
    }

    _esc(str) {
        return String(str)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;');
    }

    _buildPopoverContent(meta) {
        const e = (v) => this._esc(v);
        let html = '';

        if (meta.title) {
            html += `<h3>${e(meta.title)}</h3>`;
        }
        if (meta.description) {
            html += `<p class="subtitle">${e(meta.description)}</p>`;
        }

        html += `<div class="field"><span class="label">Uploaded by</span> ${e(meta.uploaded_by || '')} on ${e(meta.uploaded_at || '')}</div>`;
        html += `<div class="field"><span class="label">Version</span> ${e(meta.version)}</div>`;
        html += `<div class="field"><span class="label">File</span> ${e(meta.filename || '')} <span style="color:#999">(${e(meta.content_type || '')})</span></div>`;

        if (meta.tags && meta.tags.length > 0) {
            const chips = meta.tags.map(t => `<span class="tag">${e(t)}</span>`).join('');
            html += `<div class="field"><span class="label">Tags</span><div class="tags">${chips}</div></div>`;
        }

        if (meta.agent) {
            const parts = [];
            if (meta.agent.model) parts.push(`&#129302; ${e(meta.agent.model)}`);
            if (meta.agent.provider) parts.push(`&#127970; ${e(meta.agent.provider)}`);
            if (meta.agent.tool) parts.push(`&#128295; ${e(meta.agent.tool)}`);
            if (parts.length > 0) {
                html += `<div class="field"><span class="label">Agent</span> ${parts.join(' &middot; ')}</div>`;
            }
        }

        if (meta.trigger) {
            html += `<div class="field"><span class="label">Trigger</span> ${e(meta.trigger)}</div>`;
        }

        if (meta.custom && Object.keys(meta.custom).length > 0) {
            for (const [k, v] of Object.entries(meta.custom)) {
                html += `<div class="field"><span class="label">${e(k)}</span>: ${e(v)}</div>`;
            }
        }

        html += '<hr class="separator">';
        html += `<div class="field"><a href="${e(meta.url || '')}">&#128196; Rendered</a> &nbsp; <a href="${e(meta.raw_url || '')}">&#128196; Raw</a></div>`;

        return html;
    }
}

customElements.define('agentbin-badge', AgentbinBadge);
