# kotoba

App de escritorio para estudiar japonés con repasos espaciados que aparecen solos, sin exigir sesiones largas. Este glosario fija el vocabulario de repaso, madurez y progreso.

## Repaso

**Repaso**:
Respuesta del usuario a una palabra que ya toca practicar.
_Avoid_: prueba, test, tarjeta

**Intervalo de repaso**:
Tiempo que pasa desde un repaso hasta que esa palabra vuelve a estar disponible.
_Avoid_: vencimiento, plazo

**Palabra nueva**:
Palabra que todavía no se ha repasado nunca.
_Avoid_: palabra pendiente, palabra sin ver

## Madurez de una palabra

**Madurez**:
Estado de una palabra según su intervalo actual: nueva, aprendiendo, reaprendiendo, joven o madura.
_Avoid_: maestría, nivel

**Palabra aprendiendo**:
Palabra que ya se ha repasado pero aún no ha alcanzado un intervalo de un día, y que nunca lo había alcanzado antes.
_Avoid_: palabra en curso, palabra débil

**Palabra reaprendiendo**:
Palabra que ya había alcanzado al menos un día de intervalo y, tras un fallo, vuelve a estar por debajo de un día.
_Avoid_: palabra fallada, palabra olvidada

**Palabra joven**:
Palabra cuyo intervalo actual está entre uno y veinte días.
_Avoid_: palabra intermedia

**Palabra madura**:
Palabra cuyo intervalo actual es de veintiún días o más.
_Avoid_: palabra dominada, palabra consolidada

## Dominio de una lección

**Lección**:
Agrupación de palabras importada del mazo, como un capítulo o un nivel.
_Avoid_: mazo, categoría

**Patrón**:
Ítem que representa una estructura gramatical en lugar de una palabra suelta. Solo entra en los repasos si el ajuste de patrones está activo.
_Avoid_: regla, gramática

**Dominio**:
Porcentaje de palabras maduras sobre el total de palabras de la lección que entran en los repasos.
_Avoid_: maestría, progreso

**Umbral de dominio**:
Porcentaje mínimo de palabras maduras que debe alcanzar una lección para considerarse dominada.
_Avoid_: umbral de maestría

**Lección dominada**:
Lección sin palabras nuevas cuyo dominio alcanza el umbral configurado.
_Avoid_: lección maestra, lección completada
