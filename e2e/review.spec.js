import assert from 'node:assert/strict';
import path from 'node:path';

describe('desktop review persistence', () => {
  it('imports a deck, records an answer, and restores it after restart', async () => {
    const assertHidden = async () => {
      const visible = await browser.executeAsync(async (done) => {
        done(await window.__TAURI__.window.getCurrentWindow().isVisible());
      });
      assert.equal(visible, false, 'The test window must stay hidden');
    };
    await assertHidden();
    if (process.env.KOTOBA_E2E_PHASE === 'import') {
      await $('.dropzone').waitForDisplayed({ timeout: 30000 });
      const fixture = path.resolve('e2e/deck.txt');
      await browser.execute((selected) => {
        window.__KOTOBA_E2E_DECK = selected;
      }, fixture);
      await $('.dropzone').click();
      const confirm = await $('button*=Importar con este mapeo');
      await confirm.waitForEnabled({ timeout: 30000 });
      for (const [label, value] of [['Lectura', '0'], ['Significado', '1'], ['Kanji', '']]) {
        await browser.execute((name, choice) => {
          const select = document.querySelector(`select[aria-label="${name}"]`);
          select.value = choice;
          select.dispatchEvent(new Event('change', { bubbles: true }));
        }, label, value);
      }
      await confirm.click();
      const start = await $('button*=Empezar a repasar');
      await start.waitForDisplayed({ timeout: 30000 });
      await start.click();
      await $('button*=Ajustes').click();
      const alternate = await $('button=Alternar');
      await alternate.waitForDisplayed({ timeout: 30000 });
      await alternate.click();
      await browser.waitUntil(async () => (await alternate.getAttribute('aria-pressed')) === 'true');
      const save = await $('button=Guardar cambios');
      await save.click();
      await browser.waitUntil(async () => {
        const settings = await browser.executeAsync(async (done) => {
          try { done(await window.__TAURI__.core.invoke('get_settings')); }
          catch (error) { done({ error: String(error) }); }
        });
        return settings.direction_mode === 'alternate';
      }, { timeout: 30000, timeoutMsg: 'Direction setting was not persisted' });
      await $('button=Practicar').click();
      const input = await $('.answer-input');
      await input.waitForDisplayed({ timeout: 30000 });
      const prompt = await $('.prompt-jp');
      assert.equal(await prompt.getText(), 'ねこ');
      await input.setValue('cat');
      await $('button[type="submit"]').click();
      await $('.feedback.ok').waitForDisplayed({ timeout: 30000 });
      await assertHidden();
    } else {
      const progress = await $('button*=Progreso');
      await progress.waitForDisplayed({ timeout: 30000 });
      await progress.click();
      const row = await $('.history-table tbody tr');
      await row.waitForDisplayed({ timeout: 30000 });
      assert.match(await row.getText(), /ねこ/);
      assert.match(await row.getText(), /cat/);
      assert.match(await row.getText(), /Acierto/);
      await assertHidden();
    }
  });
});
