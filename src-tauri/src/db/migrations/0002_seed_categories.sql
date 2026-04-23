INSERT INTO industry_categories (name_de, osm_tags, probability_weight, enabled, color, sort_order) VALUES
('Logistik / Spedition',          '[{"office":"logistics"},{"shop":"wholesale"}]',                 95, 1, '#ef4444', 10),
('Lebensmittel-Großhandel',       '[{"shop":"wholesale","wholesale":"food"}]',                     90, 1, '#f97316', 20),
('Lagerhalle / Warehouse',        '[{"industrial":"warehouse"},{"building":"warehouse"}]',         85, 1, '#f59e0b', 30),
('Industrielle Produktion',       '[{"building":"industrial"},{"landuse":"industrial"}]',          80, 1, '#eab308', 40),
('Baumarkt / DIY',                '[{"shop":"doityourself"},{"shop":"hardware"}]',                 80, 1, '#84cc16', 45),
('Lebensmittel-Einzelhandel',     '[{"shop":"supermarket"},{"shop":"convenience"}]',               75, 1, '#22c55e', 50),
('Möbel-/Bauhandel',              '[{"shop":"furniture"},{"shop":"trade"}]',                       70, 1, '#10b981', 60),
('Pharma / Kosmetik',             '[{"industrial":"chemical"}]',                                   65, 1, '#06b6d4', 70),
('Bäckerei (industriell)',        '[{"craft":"bakery"},{"shop":"bakery"}]',                        60, 1, '#3b82f6', 80),
('Autohaus',                      '[{"shop":"car"}]',                                              40, 1, '#8b5cf6', 90),
('Bürogebäude',                   '[{"building":"office"},{"office":"company"}]',                   5, 0, '#a855f7', 100);
